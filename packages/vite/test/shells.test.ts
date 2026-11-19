// @vitest-environment node
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { VERSION } from "@weft-ui/runtime/protocol";
import { expect, test, vi } from "vitest";
import { checkShellProtocol, nativeShell } from "../src/shells.ts";

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

test("a protocol mismatch identifies both packages and protocol versions", () => {
  const shell = mkdtempSync(join(tmpdir(), "weft-shell-"));
  const protocol = VERSION + 1;
  try {
    writeFileSync(join(shell, "package.json"), JSON.stringify({ name: "@weft-ui/native-android", weft: { protocol } }));
    expect(() => checkShellProtocol("android", shell)).toThrow(
      `weft: protocol mismatch for android: @weft-ui/native-android uses ${protocol}, but @weft-ui/runtime uses ${VERSION}.`,
    );
  } finally {
    rmSync(shell, { recursive: true, force: true });
  }
});
