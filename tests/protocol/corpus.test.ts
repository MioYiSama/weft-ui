// The shared protocol corpus (protocol/corpus) against the JS encoder and decoder.
import { readFileSync } from "node:fs";
import { describe, expect, test } from "vitest";
import type { Value } from "@weft-ui/runtime/remote";
import { type Op, OpWriter, OpsError, decode } from "../../packages/runtime/src/ops.ts";

type CorpusValue = { bool: boolean } | { number: number } | { string: string };
type CorpusOp = Exclude<Op, { op: "set" }> | { op: "set"; scope: number; slot: number; value: CorpusValue };
interface OpsCase {
  name: string;
  hex: string;
  ops?: CorpusOp[];
  error?: OpsError["kind"];
}
interface ValueCase {
  name: string;
  value: CorpusValue;
  hex: string;
}

function corpus<T>(file: string): T {
  return JSON.parse(readFileSync(new URL(`../../protocol/corpus/${file}`, import.meta.url), "utf8")) as T;
}

function bytes(hex: string): Uint8Array {
  return Uint8Array.from(hex.replace(/\s+/g, "").match(/../g) ?? [], (byte) => parseInt(byte, 16));
}

function value(value: CorpusValue): Value {
  return "bool" in value ? value.bool : "number" in value ? value.number : value.string;
}

const { cases: opsCases } = corpus<{ cases: OpsCase[] }>("ops.json");
const { cases: valueCases } = corpus<{ cases: ValueCase[] }>("values.json");
const valid = opsCases.filter((c) => c.ops !== undefined);

describe("ops", () => {
  test.each(opsCases.map((c) => [c.name, c] as const))("decodes %s", (_, c) => {
    if (c.ops) {
      expect(decode(bytes(c.hex))).toEqual(c.ops.map((o) => (o.op === "set" ? { ...o, value: value(o.value) } : o)));
    } else {
      let error: unknown;
      try {
        decode(bytes(c.hex));
      } catch (e) {
        error = e;
      }
      expect(error).toBeInstanceOf(OpsError);
      expect((error as OpsError).kind).toBe(c.error);
    }
  });

  test.each(valid.map((c) => [c.name, c] as const))("encodes %s", (_, c) => {
    const writer = new OpWriter();
    for (const o of decode(bytes(c.hex))) {
      switch (o.op) {
        case "create":
          writer.create(o.scope, o.template);
          break;
        case "set":
          writer.set(o.scope, o.slot, o.value);
          break;
        case "branch":
          writer.branch(o.scope, o.point, o.child);
          break;
        case "rows":
          writer.rows(o.scope, o.point, o.children);
          break;
        case "attach":
          writer.attach(o.scope, o.point, o.child);
          break;
      }
    }
    expect(writer.take()).toEqual(bytes(c.hex));
  });
});

test.each(valueCases.map((c) => [c.name, c] as const))("encodes the value %s", (_, c) => {
  const writer = new OpWriter();
  writer.set(0, 0, value(c.value));
  // Version, opcode and the scope and slot u32s precede the value.
  expect(writer.take()?.subarray(10)).toEqual(bytes(c.hex));
});
