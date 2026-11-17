import { view } from "@weft-ui/runtime/wechat";
import { PlatformError } from "./error.ts";
import type { Api, App as WeftApp, RequestInit, Response } from "./index.d.ts";

export { PlatformError } from "./error.ts";
export type { Api, App, Platform, RequestInit, Response } from "./index.d.ts";

interface Failure {
  errMsg: string;
}

interface Wx {
  request(options: {
    url: string;
    method?: string;
    header?: Record<string, string>;
    data?: string | ArrayBuffer;
    responseType: "arraybuffer";
    success(result: { statusCode: number; data: ArrayBuffer }): void;
    fail(error: Failure): void;
  }): void;
  getStorage(options: {
    key: string;
    success(result: { data: string }): void;
    fail(error: Failure): void;
  }): void;
  setStorage(options: {
    key: string;
    data: string;
    success(): void;
    fail(error: Failure): void;
  }): void;
  removeStorage(options: {
    key: string;
    success(): void;
    fail(error: Failure): void;
  }): void;
  getAppBaseInfo(): { version: string };
}

declare const wx: Wx;
declare function App(options: {
  weft: {
    view: typeof view;
    mount(target: never): void;
    unmount(): void;
  };
}): void;

// Decode UTF-8 without relying on TextDecoder in mini program logic layers.
// Invalid sequences use U+FFFD, consuming only their valid prefix.
function decodeUtf8(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let text = "";
  let i = bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf ? 3 : 0;
  while (i < bytes.length) {
    const first = bytes[i++];
    if (first < 0x80) {
      text += String.fromCharCode(first);
      continue;
    }
    let remaining: number;
    let point: number;
    let lower = 0x80;
    let upper = 0xbf;
    if (first >= 0xc2 && first <= 0xdf) {
      remaining = 1;
      point = first & 0x1f;
    } else if (first >= 0xe0 && first <= 0xef) {
      remaining = 2;
      point = first & 0x0f;
      if (first === 0xe0) lower = 0xa0;
      if (first === 0xed) upper = 0x9f;
    } else if (first >= 0xf0 && first <= 0xf4) {
      remaining = 3;
      point = first & 0x07;
      if (first === 0xf0) lower = 0x90;
      if (first === 0xf4) upper = 0x8f;
    } else {
      text += "\ufffd";
      continue;
    }
    while (remaining > 0 && i < bytes.length && bytes[i] >= lower && bytes[i] <= upper) {
      point = (point << 6) | (bytes[i++] & 0x3f);
      remaining--;
      lower = 0x80;
      upper = 0xbf;
    }
    text += remaining === 0 ? String.fromCodePoint(point) : "\ufffd";
  }
  return text;
}

function response(status: number, data: ArrayBuffer): Response {
  let decoded: string | undefined;

  return {
    status,
    ok: status >= 200 && status < 300,
    async text() { return decoded ??= decodeUtf8(data); },
    async json() { return JSON.parse(decoded ??= decodeUtf8(data)) as unknown; },
    async arrayBuffer() { return data; },
  };
}

/** Registers the app lifecycle and generated-view factory with the WeChat host. */
export function run(app: WeftApp): void {
  App({
    weft: {
      view,
      mount: (target) => app.mount(target),
      unmount: () => app.unmount(),
    },
  });
}

/** Sends a binary request through wx.request and exposes portable body readers. */
export async function fetch(url: string, init?: RequestInit): Promise<Response> {
  if (!isSupported("fetch")) throw new PlatformError("fetch", "wechat");
  return new Promise((resolve, reject) => {
    wx.request({
      url,
      method: init?.method,
      header: init?.headers,
      data: init?.body,
      responseType: "arraybuffer",
      success: ({ statusCode, data }) => resolve(response(statusCode, data)),
      fail: ({ errMsg }) => reject(new TypeError(errMsg)),
    });
  });
}

/** Promise-based access to WeChat's persistent string storage. */
export const storage = {
  async get(key: string): Promise<string | null> {
    if (!isSupported("storage")) throw new PlatformError("storage", "wechat");
    return new Promise((resolve, reject) => {
      wx.getStorage({
        key,
        success: ({ data }) => resolve(data),
        fail: ({ errMsg }) => {
          if (errMsg.includes("data not found")) resolve(null);
          else reject(new Error(errMsg));
        },
      });
    });
  },
  async set(key: string, value: string): Promise<void> {
    if (!isSupported("storage")) throw new PlatformError("storage", "wechat");
    return new Promise((resolve, reject) => {
      wx.setStorage({
        key,
        data: value,
        success: () => resolve(),
        fail: ({ errMsg }) => reject(new Error(errMsg)),
      });
    });
  },
  async remove(key: string): Promise<void> {
    if (!isSupported("storage")) throw new PlatformError("storage", "wechat");
    return new Promise((resolve, reject) => {
      wx.removeStorage({
        key,
        success: () => resolve(),
        fail: ({ errMsg }) => reject(new Error(errMsg)),
      });
    });
  },
};

/** WeChat host version, read only when requested so imports do not access wx. */
export const platform = {
  os: "wechat",
  get version(): string { return wx.getAppBaseInfo().version; },
} as const;

/** Checks whether all host functions for the requested capability exist. */
export function isSupported(api: Api): boolean {
  if (typeof wx === "undefined") return false;
  return api === "fetch"
    ? typeof wx.request === "function"
    : typeof wx.getStorage === "function"
      && typeof wx.setStorage === "function"
      && typeof wx.removeStorage === "function";
}
