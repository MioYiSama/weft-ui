// Fetch frames of the shared protocol corpus (protocol/corpus/frames.json).
import { readFileSync } from "node:fs";
import { expect, test } from "vitest";
import { frame, unframe } from "../src/frame.ts";

interface Frame {
  name: string;
  head: unknown;
  body: string;
  hex: string;
}

const corpus = JSON.parse(readFileSync(new URL("../../../protocol/corpus/frames.json", import.meta.url), "utf8")) as {
  requests: Frame[];
  responses: Frame[];
  malformed: { name: string; hex: string }[];
};

function bytes(hex: string): Uint8Array {
  return Uint8Array.from(hex.replace(/\s+/g, "").match(/../g) ?? [], (byte) => parseInt(byte, 16));
}

test.each(corpus.requests.map((c) => [c.name, c] as const))("frames the request %s", (_, c) => {
  expect(frame(c.head, c.body)).toEqual(bytes(c.hex));
});

test.each(corpus.responses.map((c) => [c.name, c] as const))("unframes the response %s", (_, c) => {
  const { head, body } = unframe(bytes(c.hex).buffer as ArrayBuffer);
  expect(head).toEqual(c.head);
  expect(new TextDecoder().decode(body)).toBe(c.body);
});

test.each(corpus.malformed.map((c) => [c.name, c] as const))("rejects the malformed frame %s", (_, c) => {
  expect(() => unframe(bytes(c.hex).buffer as ArrayBuffer)).toThrow();
});
