/**
 * Native platforms (iOS, macOS, Android, Windows, Linux): every API is a
 * platform module of the shell, reached through the kernel's `__weft.call`.
 * The wire format of each module is specified in
 * docs/src/content/docs/protocol.md ("Platform calls").
 */

import { NativeSink, kernel } from "@weft-ui/runtime/native";
import { PlatformError } from "./error.ts";
import type { Api, App, RequestInit, Response } from "./index.d.ts";

export { PlatformError } from "./error.ts";
export type { Api, App, Platform, RequestInit, Response } from "./index.d.ts";

const os = __WEFT_PLATFORM__;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** Mounts the app into the native views of the kernel that runs the bundle. */
export function run(app: App): void {
  app.mount(new NativeSink() as never);
}

/** A JSON head and raw body, framed as `u32 head length (LE) | head (UTF-8) | body`. */
function frame(head: unknown, body?: string | ArrayBuffer): Uint8Array {
  const json = encoder.encode(JSON.stringify(head));
  const rest = body === undefined ? new Uint8Array(0) : typeof body === "string" ? encoder.encode(body) : new Uint8Array(body);
  const bytes = new Uint8Array(4 + json.length + rest.length);
  new DataView(bytes.buffer).setUint32(0, json.length, true);
  bytes.set(json, 4);
  bytes.set(rest, 4 + json.length);
  return bytes;
}

function unframe(buffer: ArrayBuffer): { head: unknown; body: ArrayBuffer } {
  const length = new DataView(buffer).getUint32(0, true);
  return {
    head: JSON.parse(decoder.decode(new Uint8Array(buffer, 4, length))),
    body: buffer.slice(4 + length),
  };
}

/** Sends a request through the shell's `fetch` module. */
export async function fetch(url: string, init?: RequestInit): Promise<Response> {
  if (!isSupported("fetch")) throw new PlatformError("fetch", os);
  const head = { url, method: init?.method ?? "GET", headers: init?.headers ?? {} };
  const { head: result, body } = unframe(await kernel().call("fetch", "request", frame(head, init?.body)));
  const { status } = result as { status: number };
  let text: string | undefined;
  return {
    status,
    ok: status >= 200 && status < 300,
    async text() { return text ??= decoder.decode(body); },
    async json() { return JSON.parse(text ??= decoder.decode(body)) as unknown; },
    async arrayBuffer() { return body; },
  };
}

/** Calls a `storage` method with JSON arguments and returns its JSON result (`undefined` if empty). */
async function store(method: string, args: string[]): Promise<unknown> {
  if (!isSupported("storage")) throw new PlatformError("storage", os);
  const result = await kernel().call("storage", method, JSON.stringify(args));
  return result.byteLength === 0 ? undefined : JSON.parse(decoder.decode(result));
}

/** Persistent string storage of the shell's `storage` module. */
export const storage = {
  async get(key: string): Promise<string | null> {
    return (await store("get", [key])) as string | null;
  },
  async set(key: string, value: string): Promise<void> {
    await store("set", [key, value]);
  },
  async remove(key: string): Promise<void> {
    await store("remove", [key]);
  },
};

/** The native platform and the OS version reported by the shell. */
export const platform = {
  os,
  get version(): string { return globalThis.__weft?.version ?? ""; },
};

/** Whether the shell implements the API's platform module. */
export function isSupported(api: Api): boolean {
  return globalThis.__weft?.supports(api) ?? false;
}
