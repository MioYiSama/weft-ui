/** Platforms recognized by Weft; availability depends on the selected build target. */
export type Platform = "web" | "wechat" | "ios" | "macos" | "android" | "windows" | "linux";

declare global {
  /** The platform being built, defined by `@weft-ui/vite` in every environment. */
  const __WEFT_PLATFORM__: Platform;
}

/** Optional platform capabilities. */
export type Api = "fetch" | "storage";

/** An API unavailable on the current platform. Async APIs reject with this error. */
export class PlatformError extends Error {
  /** The unsupported capability. */
  readonly api: Api;
  constructor(api: Api, os: Platform);
}

/** The lifecycle shared by generated root components on every platform. */
export interface App {
  /** Mounts into the platform-specific target supplied by run. */
  mount(target: never): void;
  /** Releases the mounted app. */
  unmount(): void;
}

/** Starts an app using the selected platform's host lifecycle. */
export function run(app: App): void;

/** Portable HTTP request options. */
export interface RequestInit {
  /** HTTP method; the host defaults to GET. */
  method?: string;
  /** Request headers. */
  headers?: Record<string, string>;
  /** Request payload. */
  body?: string | ArrayBuffer;
}

/** An HTTP response whose body can be read in any of the portable formats. */
export interface Response {
  /** Whether the HTTP status is in the range 200–299. */
  readonly ok: boolean;
  /** HTTP status code. */
  readonly status: number;
  /** Reads the body as UTF-8 text. */
  text(): Promise<string>;
  /** Parses the body as JSON. */
  json(): Promise<unknown>;
  /** Reads the raw response bytes. */
  arrayBuffer(): Promise<ArrayBuffer>;
}

/** Sends a request, rejecting with PlatformError if fetch is unsupported. */
export function fetch(url: string, init?: RequestInit): Promise<Response>;

/** Persistent string storage; methods reject with PlatformError when unsupported. */
export const storage: {
  /** Reads a key, returning null when it does not exist. */
  get(key: string): Promise<string | null>;
  /** Stores a string value. */
  set(key: string, value: string): Promise<void>;
  /** Removes a key. */
  remove(key: string): Promise<void>;
};

/** Current operating platform and host version (the user agent on the web). */
export const platform: { readonly os: Platform; readonly version: string };

/** Reports current host capability availability, without invoking the API. */
export function isSupported(api: Api): boolean;
