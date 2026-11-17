import { view } from "@weft-ui/runtime/wechat";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { fetch, isSupported, platform, PlatformError, run, storage } from "../src/wechat.ts";
import { PlatformError as SharedPlatformError } from "../src/error.ts";

interface Failure { errMsg: string }
interface RequestOptions {
  url: string;
  method?: string;
  header?: Record<string, string>;
  data?: string | ArrayBuffer;
  responseType: string;
  success(result: { statusCode: number; data: ArrayBuffer }): void;
  fail(error: Failure): void;
}

function createWx() {
  const values = new Map<string, string>();
  return {
    values,
    request: vi.fn((options: RequestOptions) => {
      options.success({ statusCode: 200, data: new ArrayBuffer(0) });
    }),
    getStorage: vi.fn((options: {
      key: string;
      success(result: { data: string }): void;
      fail(error: Failure): void;
    }) => {
      if (values.has(options.key)) options.success({ data: values.get(options.key)! });
      else options.fail({ errMsg: "getStorage:fail data not found" });
    }),
    setStorage: vi.fn((options: {
      key: string;
      data: string;
      success(): void;
      fail(error: Failure): void;
    }) => {
      values.set(options.key, options.data);
      options.success();
    }),
    removeStorage: vi.fn((options: {
      key: string;
      success(): void;
      fail(error: Failure): void;
    }) => {
      values.delete(options.key);
      options.success();
    }),
    getAppBaseInfo: vi.fn(() => ({ version: "8.0.65" })),
  };
}

let wx = createWx();

beforeEach(() => {
  wx = createWx();
  vi.stubGlobal("wx", wx);
  vi.stubGlobal("App", vi.fn());
});

afterEach(() => {
  vi.unstubAllGlobals();
});

test.each(["string", "binary"])("request maps headers and a %s body and returns raw bytes", async (kind) => {
  const bytes = new Uint8Array([0, 128, 255]).buffer;
  wx.request.mockImplementation(({ success }) => success({ statusCode: 201, data: bytes }));
  const body = kind === "string" ? "payload" : new Uint8Array([1, 2]).buffer;
  const headers = { "Content-Type": "application/octet-stream", Authorization: "Bearer token" };
  const response = await fetch("https://example.test/upload", { method: "POST", headers, body });

  expect(isSupported("fetch")).toBe(true);
  expect(wx.request).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({
    url: "https://example.test/upload", method: "POST", header: headers, data: body,
    responseType: "arraybuffer",
  }));
  expect(wx.request.mock.calls[0][0].data).toBe(body);
  expect(response.status).toBe(201);
  expect(response.ok).toBe(true);
  expect(await response.arrayBuffer()).toBe(bytes);
});

test.each([
  { bytes: [0x41, 0xc3, 0xa9, 0xe4, 0xb8, 0xad, 0xf0, 0x9f, 0x8c, 0x8d], text: "Aé中🌍" },
  { bytes: [0xef, 0xbb, 0xbf, 0x41], text: "A" },
  { bytes: [0xe2, 0x82], text: "�" },
  { bytes: [0xe2, 0x82, 0x41, 0x80], text: "�A�" },
  { bytes: [0xc0, 0xaf, 0xed, 0xa0, 0x80, 0xf4, 0x90, 0x80, 0x80], text: "���������" },
  { bytes: [], text: "" },
])("decodes UTF-8 without TextDecoder: $text", async ({ bytes, text }) => {
  vi.stubGlobal("TextDecoder", undefined);
  wx.request.mockImplementation(({ success }) => success({ statusCode: 200, data: new Uint8Array(bytes).buffer }));
  const response = await fetch("https://example.test/text");
  expect(await response.text()).toBe(text);
  expect(await response.text()).toBe(text);
});

test("reads JSON from UTF-8 bytes and rejects malformed JSON", async () => {
  const bytes = new TextEncoder().encode('{"message":"你好 🌍"}').buffer;
  wx.request.mockImplementation(({ success }) => success({ statusCode: 200, data: bytes }));
  const response = await fetch("https://example.test/json");
  expect(await response.json()).toEqual({ message: "你好 🌍" });
  expect(await response.text()).toBe('{"message":"你好 🌍"}');

  wx.request.mockImplementation(({ success }) => success({ statusCode: 200, data: new Uint8Array([0x7b]).buffer }));
  await expect((await fetch("https://example.test/invalid")).json()).rejects.toBeInstanceOf(SyntaxError);
});

test.each([199, 200, 204, 299, 300, 404, 500])("reports status %i without rejecting HTTP errors", async (statusCode) => {
  wx.request.mockImplementation(({ success }) => success({ statusCode, data: new ArrayBuffer(0) }));
  const response = await fetch("https://example.test/status");
  expect(response.status).toBe(statusCode);
  expect(response.ok).toBe(statusCode >= 200 && statusCode < 300);
});

test("network failure rejects with a TypeError carrying the host message", async () => {
  wx.request.mockImplementation(({ fail }) => fail({ errMsg: "request:fail timeout" }));
  const result = fetch("https://example.test/slow");
  await expect(result).rejects.toBeInstanceOf(TypeError);
  await expect(result).rejects.toThrow("request:fail timeout");
});

test("storage handles missing keys and round-trips and removes strings", async () => {
  expect(isSupported("storage")).toBe(true);
  expect(await storage.get("theme")).toBeNull();
  await storage.set("theme", "深色 🌙");
  expect(wx.values.get("theme")).toBe("深色 🌙");
  expect(await storage.get("theme")).toBe("深色 🌙");
  expect(wx.getStorage).toHaveBeenLastCalledWith(expect.objectContaining({ key: "theme" }));
  expect(wx.setStorage).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ key: "theme", data: "深色 🌙" }));
  await storage.remove("theme");
  expect(wx.removeStorage).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({ key: "theme" }));
  expect(wx.values.has("theme")).toBe(false);
  expect(await storage.get("theme")).toBeNull();
});

test.each(["get", "set", "remove"] as const)("storage.%s propagates non-missing host failures", async (method) => {
  const hostMethod = { get: "getStorage", set: "setStorage", remove: "removeStorage" }[method];
  vi.stubGlobal("wx", {
    ...wx,
    [hostMethod]: ({ fail }: { fail(error: Failure): void }) => fail({ errMsg: "storage:fail permission denied" }),
  });
  const result = method === "set" ? storage.set("key", "value") : storage[method]("key");
  await expect(result).rejects.toThrow("storage:fail permission denied");
});

test("run registers the runtime view factory and forwards mount/unmount", () => {
  const register = vi.fn();
  vi.stubGlobal("App", register);
  const app = { mount: vi.fn(), unmount: vi.fn() };
  run(app);
  expect(register).toHaveBeenCalledOnce();
  const { weft } = register.mock.calls[0][0];
  expect(weft.view).toBe(view);
  const target = { setData: vi.fn() };
  weft.mount(target);
  weft.unmount();
  expect(app.mount).toHaveBeenCalledExactlyOnceWith(target);
  expect(app.unmount).toHaveBeenCalledExactlyOnceWith();
});

test("imports without wx and reads the host version lazily", async () => {
  vi.stubGlobal("wx", undefined);
  vi.resetModules();
  // A fresh import is necessary to exercise the no-wx module-loading boundary.
  const imported = await import("../src/wechat.ts");
  expect(imported.platform.os).toBe("wechat");
  vi.stubGlobal("wx", wx);
  expect(wx.getAppBaseInfo).not.toHaveBeenCalled();
  expect(imported.platform.version).toBe("8.0.65");
  wx.getAppBaseInfo.mockReturnValue({ version: "8.0.66" });
  expect(imported.platform.version).toBe("8.0.66");
});

test("missing wx rejects unsupported async APIs with the shared PlatformError", async () => {
  vi.stubGlobal("wx", undefined);
  expect(isSupported("fetch")).toBe(false);
  expect(isSupported("storage")).toBe(false);
  for (const [api, result] of [
    ["fetch", fetch("https://example.test")],
    ["storage", storage.get("key")],
    ["storage", storage.set("key", "value")],
    ["storage", storage.remove("key")],
  ] as const) {
    expect(result).toBeInstanceOf(Promise);
    await expect(result).rejects.toBeInstanceOf(PlatformError);
    await expect(result).rejects.toBeInstanceOf(SharedPlatformError);
    await expect(result).rejects.toMatchObject({
      api, name: "PlatformError", message: `${api} is not supported on wechat`,
    });
  }
});

test.each(["request", "getStorage", "setStorage", "removeStorage"] as const)("detects missing wx.%s", async (method) => {
  vi.stubGlobal("wx", { ...wx, [method]: undefined });
  const api = method === "request" ? "fetch" : "storage";
  expect(isSupported(api)).toBe(false);
  const result = api === "fetch" ? fetch("https://example.test") : storage.get("key");
  await expect(result).rejects.toBeInstanceOf(PlatformError);
});
