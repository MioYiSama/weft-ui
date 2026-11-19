/**
 * Binary encoding of view protocol ops (protocol/SPEC.md),
 * its decoder, and the one-line text form of an op used by dumps and golden
 * files.
 */

import { VERSION, Opcode, ValueTag, type OpErrorKind } from "./protocol.ts";
import type { Ops, Value } from "./remote.ts";

/**
 * A batch that failed to decode; `kind` is one of the error kinds of
 * protocol/SPEC.md ("Decoding errors").
 */
export class OpsError extends Error {
  readonly kind: OpErrorKind;

  constructor(kind: OpErrorKind, message: string) {
    super(message);
    this.name = "OpsError";
    this.kind = kind;
  }
}

/** A decoded op. */
export type Op =
  | { op: "create"; scope: number; template: number }
  | { op: "set"; scope: number; slot: number; value: Value }
  | { op: "branch"; scope: number; point: number; child: number }
  | { op: "rows"; scope: number; point: number; children: number[] }
  | { op: "attach"; scope: number; point: number; child: number };

/** Writes ops into one reusable buffer; `take` hands out each batch. */
export class OpWriter implements Ops {
  #bytes = new Uint8Array(256);
  #view = new DataView(this.#bytes.buffer);
  #length = 0;
  #encoder = new TextEncoder();

  /** Whether no op was written since the last `take`. */
  get empty(): boolean {
    return this.#length === 0;
  }

  /**
   * Returns the batch written since the last call and starts a new one, or
   * `null` if it is empty. The bytes are only valid until the next write.
   */
  take(): Uint8Array | null {
    if (this.#length === 0) return null;
    const batch = this.#bytes.subarray(0, this.#length);
    this.#length = 0;
    return batch;
  }

  create(scope: number, template: number): void {
    this.#op(Opcode.Create, 8);
    this.#u32(scope);
    this.#u32(template);
  }

  set(scope: number, slot: number, value: Value): void {
    if (typeof value === "string") {
      // UTF-8 needs at most three bytes per UTF-16 code unit.
      this.#op(Opcode.Set, 13 + value.length * 3);
      this.#u32(scope);
      this.#u32(slot);
      this.#bytes[this.#length++] = ValueTag.String;
      const start = this.#length + 4;
      const { written } = this.#encoder.encodeInto(value, this.#bytes.subarray(start));
      this.#u32(written);
      this.#length += written;
    } else if (typeof value === "number") {
      this.#op(Opcode.Set, 17);
      this.#u32(scope);
      this.#u32(slot);
      this.#bytes[this.#length++] = ValueTag.Number;
      this.#view.setFloat64(this.#length, value, true);
      this.#length += 8;
    } else {
      this.#op(Opcode.Set, 9);
      this.#u32(scope);
      this.#u32(slot);
      this.#bytes[this.#length++] = value ? ValueTag.True : ValueTag.False;
    }
  }

  branch(scope: number, point: number, child: number): void {
    this.#op(Opcode.Branch, 12);
    this.#u32(scope);
    this.#u32(point);
    this.#u32(child);
  }

  rows(scope: number, point: number, children: readonly number[]): void {
    this.#op(Opcode.Rows, 12 + children.length * 4);
    this.#u32(scope);
    this.#u32(point);
    this.#u32(children.length);
    for (const child of children) this.#u32(child);
  }

  attach(scope: number, point: number, child: number): void {
    this.#op(Opcode.Attach, 12);
    this.#u32(scope);
    this.#u32(point);
    this.#u32(child);
  }

  /** Writes the batch header if needed and the opcode, reserving `size` more bytes. */
  #op(code: number, size: number): void {
    this.#reserve(size + 2);
    if (this.#length === 0) this.#bytes[this.#length++] = VERSION;
    this.#bytes[this.#length++] = code;
  }

  #reserve(size: number): void {
    const needed = this.#length + size;
    if (needed <= this.#bytes.length) return;
    let capacity = this.#bytes.length * 2;
    while (capacity < needed) capacity *= 2;
    const bytes = new Uint8Array(capacity);
    bytes.set(this.#bytes.subarray(0, this.#length));
    this.#bytes = bytes;
    this.#view = new DataView(bytes.buffer);
  }

  #u32(value: number): void {
    this.#view.setUint32(this.#length, value, true);
    this.#length += 4;
  }
}

/**
 * Decodes one batch; throws an `OpsError` on truncated input, an unknown
 * version, opcode or value tag, or a string that is not well-formed UTF-8.
 */
export function decode(batch: Uint8Array): Op[] {
  const view = new DataView(batch.buffer, batch.byteOffset, batch.byteLength);
  const decoder = new TextDecoder("utf-8", { fatal: true });
  let at = 0;
  const need = (size: number) => {
    if (at + size > batch.length) throw new OpsError("truncated", `weft ops: truncated batch at byte ${at}`);
  };
  const u8 = () => {
    need(1);
    return batch[at++];
  };
  const u32 = () => {
    need(4);
    const value = view.getUint32(at, true);
    at += 4;
    return value;
  };
  const version = u8();
  if (version !== VERSION) throw new OpsError("version", `weft ops: unsupported version ${version}`);
  const ops: Op[] = [];
  while (at < batch.length) {
    const code = u8();
    switch (code) {
      case Opcode.Create:
        ops.push({ op: "create", scope: u32(), template: u32() });
        break;
      case Opcode.Set: {
        const scope = u32();
        const slot = u32();
        const tag = u8();
        let value: Value;
        if (tag === ValueTag.False || tag === ValueTag.True) {
          value = tag === ValueTag.True;
        } else if (tag === ValueTag.Number) {
          need(8);
          value = view.getFloat64(at, true);
          at += 8;
        } else if (tag === ValueTag.String) {
          const length = u32();
          need(length);
          try {
            value = decoder.decode(batch.subarray(at, at + length));
          } catch {
            throw new OpsError("utf8", `weft ops: invalid UTF-8 string at byte ${at}`);
          }
          at += length;
        } else {
          throw new OpsError("tag", `weft ops: unknown value tag ${tag} at byte ${at - 1}`);
        }
        ops.push({ op: "set", scope, slot, value });
        break;
      }
      case Opcode.Branch:
        ops.push({ op: "branch", scope: u32(), point: u32(), child: u32() });
        break;
      case Opcode.Rows: {
        const scope = u32();
        const point = u32();
        const count = u32();
        const children: number[] = [];
        for (let i = 0; i < count; i++) children.push(u32());
        ops.push({ op: "rows", scope, point, children });
        break;
      }
      case Opcode.Attach:
        ops.push({ op: "attach", scope: u32(), point: u32(), child: u32() });
        break;
      default:
        throw new OpsError("opcode", `weft ops: unknown opcode ${code} at byte ${at - 1}`);
    }
  }
  return ops;
}

/** The text form of a value: `true` / `false`, a JS number, or a JSON string. */
export function formatValue(value: Value): string {
  return typeof value === "string" ? JSON.stringify(value) : String(value);
}

/** The text form of an op, e.g. `set 3 0 "milk"` or `rows 2 0 [4 5]`. */
export function formatOp(op: Op): string {
  switch (op.op) {
    case "create":
      return `create ${op.scope} ${op.template}`;
    case "set":
      return `set ${op.scope} ${op.slot} ${formatValue(op.value)}`;
    case "branch":
      return `branch ${op.scope} ${op.point} ${op.child}`;
    case "rows":
      return `rows ${op.scope} ${op.point} [${op.children.join(" ")}]`;
    case "attach":
      return `attach ${op.scope} ${op.point} ${op.child}`;
  }
}
