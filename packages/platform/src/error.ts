import type { Api, Platform } from "./index.d.ts";

/** An API unavailable on the current platform. */
export class PlatformError extends Error {
  readonly api: Api;

  constructor(api: Api, os: Platform) {
    super(`${api} is not supported on ${os}`);
    this.name = "PlatformError";
    this.api = api;
  }
}
