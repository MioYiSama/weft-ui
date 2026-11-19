import type { Platform, PlatformModule } from "@weft-ui/runtime/protocol";

/** An API unavailable on the current platform. */
export class PlatformError extends Error {
  readonly api: PlatformModule;

  constructor(api: PlatformModule, os: Platform) {
    super(`${api} is not supported on ${os}`);
    this.name = "PlatformError";
    this.api = api;
  }
}
