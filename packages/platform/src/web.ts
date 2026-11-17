import { PlatformError } from "./error.ts";
import type { Api, App, RequestInit, Response } from "./index.d.ts";

export { PlatformError } from "./error.ts";
export type { Api, App, Platform, RequestInit, Response } from "./index.d.ts";

function localStorage(): Storage | null {
  try {
    return globalThis.localStorage ?? null;
  } catch {
    return null;
  }
}

function supportedStorage(): Storage {
  const storage = localStorage();
  if (!storage) throw new PlatformError("storage", "web");
  return storage;
}

/** Mounts a generated app into the document's #app element. */
export function run(app: App): void {
  const target = document.getElementById("app");
  if (!target) throw new Error('weft: run() requires an element with id "app"');
  app.mount(target as never);
}

/** Uses the browser's fetch implementation, rejecting when unavailable. */
export async function fetch(url: string, init?: RequestInit): Promise<Response> {
  if (!isSupported("fetch")) throw new PlatformError("fetch", "web");
  return globalThis.fetch(url, init);
}

/** Promise-based access to browser localStorage. */
export const storage = {
  async get(key: string): Promise<string | null> {
    return supportedStorage().getItem(key);
  },
  async set(key: string, value: string): Promise<void> {
    supportedStorage().setItem(key, value);
  },
  async remove(key: string): Promise<void> {
    supportedStorage().removeItem(key);
  },
};

/** Browser platform and user agent. */
export const platform = { os: "web", version: navigator.userAgent } as const;

/** Checks whether the browser currently exposes the requested capability. */
export function isSupported(api: Api): boolean {
  return api === "fetch" ? typeof globalThis.fetch === "function" : localStorage() !== null;
}
