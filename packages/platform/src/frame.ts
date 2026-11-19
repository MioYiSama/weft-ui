/**
 * Frames of the `fetch` platform module: `u32 head length (LE) | head (UTF-8
 * JSON) | body`, see docs/src/content/docs/protocol.md ("Platform calls").
 */

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** A JSON head and raw body, framed as `u32 head length (LE) | head (UTF-8) | body`. */
export function frame(head: unknown, body?: string | ArrayBuffer): Uint8Array {
  const json = encoder.encode(JSON.stringify(head));
  const rest = body === undefined ? new Uint8Array(0) : typeof body === "string" ? encoder.encode(body) : new Uint8Array(body);
  const bytes = new Uint8Array(4 + json.length + rest.length);
  new DataView(bytes.buffer).setUint32(0, json.length, true);
  bytes.set(json, 4);
  bytes.set(rest, 4 + json.length);
  return bytes;
}

/** Splits a frame into its parsed JSON head and its body; throws on a malformed frame. */
export function unframe(buffer: ArrayBuffer): { head: unknown; body: ArrayBuffer } {
  const length = new DataView(buffer).getUint32(0, true);
  return {
    head: JSON.parse(decoder.decode(new Uint8Array(buffer, 4, length))),
    body: buffer.slice(4 + length),
  };
}
