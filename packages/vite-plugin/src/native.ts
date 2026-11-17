import { execFile } from "node:child_process";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { promisify } from "node:util";
import type { Platform } from "@weft-ui/compiler";

/** Platforms whose logic runs in the native kernel (`native/core`). */
export const NATIVE: readonly Platform[] = ["ios", "macos", "android", "windows", "linux"];

/**
 * Syntax the bundle keeps for Hermes; newer syntax is lowered by Vite.
 * Measured with the test262 subset in native/core/test262 (docs/PLAN-v2.md, P3).
 */
export const HERMES_TARGET = "es2022";

/**
 * Flags of `hermesc`. Block scoping and async generators are opt-in in
 * Hermes; without block scoping `let` / `const` bindings in loops would be
 * shared by every iteration. Keep in sync with native/core/tests/CMakeLists.txt.
 */
export const HERMESC_FLAGS = ["-O", "-emit-binary", "-Xes6-block-scoping", "-Xasync-generators", "-w"];

/** `hermesc` of the `hermes-compiler` package, per host. */
const BINARIES: Partial<Record<string, string>> = {
  "darwin-arm64": "osx-bin/hermesc",
  "darwin-x64": "osx-bin/hermesc",
  "linux-x64": "linux64-bin/hermesc",
  "win32-x64": "win64-bin/hermesc.exe",
};

/**
 * The `hermesc` that compiles bundles: `WEFT_HERMESC`, else the one shipped
 * by the `hermes-compiler` package, which matches the kernel's Hermes release.
 */
export function hermesc(): string {
  const override = process.env.WEFT_HERMESC;
  if (override) return override;
  const binary = BINARIES[`${process.platform}-${process.arch}`];
  if (!binary) {
    throw new Error(
      `weft: hermes-compiler ships no hermesc for ${process.platform}-${process.arch}. ` +
        "Build it with `cmake -S native/core -B native/core/build && cmake --build native/core/build --target hermesc` " +
        "and point WEFT_HERMESC at native/core/build/bin/hermesc.",
    );
  }
  const manifest = createRequire(import.meta.url).resolve("hermes-compiler/package.json");
  return join(dirname(manifest), "hermesc", binary);
}

/** Compiles the bundle at `input` to Hermes bytecode at `output`. */
export async function compileBytecode(input: string, output: string): Promise<void> {
  const compiler = hermesc();
  try {
    await promisify(execFile)(compiler, [...HERMESC_FLAGS, `-out=${output}`, input]);
  } catch (cause) {
    const { stderr, code } = cause as { stderr?: string; code?: string };
    if (code === "ENOENT") throw new Error(`weft: hermesc not found at ${compiler}; reinstall dependencies or set WEFT_HERMESC`);
    throw new Error(`weft: hermesc failed on ${input}\n${stderr ?? String(cause)}`);
  }
}
