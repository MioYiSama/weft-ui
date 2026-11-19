// @vitest-environment node
import { join } from "node:path";
import { expect, test, vi } from "vitest";
import { nativeShell } from "../src/shells.ts";

vi.mock("node:module", () => ({
  createRequire: () => ({
    resolve() {
      throw Object.assign(new Error("Cannot find module"), { code: "MODULE_NOT_FOUND" });
    },
  }),
}));

test("a missing native shell gives the install command for its platform", () => {
  expect(() => nativeShell("ios")).toThrow(
    "weft: building for ios needs the native shell; install it with `pnpm add -D @weft-ui/native-apple`",
  );
});

test("a native root override does not require an installed shell package", () => {
  expect(nativeShell("macos", "custom")).toBe(join("custom", "apple"));
});
