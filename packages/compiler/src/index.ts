import { readFileSync } from "node:fs";
import type * as WasmCompiler from "../wasm/weft_ui_wasm.js";

let compiler: Promise<typeof WasmCompiler> | undefined;

function loadCompiler() {
  compiler ??= (async () => {
    let bytes: Buffer;
    try {
      bytes = readFileSync(new URL("../wasm/weft_ui_wasm_bg.wasm", import.meta.url));
    } catch (cause) {
      throw new Error("Weft compiler WASM is unavailable. Run `pnpm build:wasm` from the workspace root.", { cause });
    }
    // Generated glue may not exist before build:wasm; defer loading so the error above explains the fix.
    const module = await import("../wasm/weft_ui_wasm.js");
    module.initSync({ module: bytes });
    return module;
  })();
  return compiler;
}

/** Target platform; matches `WEFT_PLATFORM`. */
export type Platform = "web" | "wechat" | "ios" | "macos" | "android" | "windows" | "linux";

/** Generated JS logic module of one `.weft` file. */
export interface LogicModule {
  /** Normalized source path, as the compiler loaded it. */
  source: string;
  code: string;
  dts: string;
}

/** Generated view file, relative to the platform's output root. */
export interface OutputFile {
  path: string;
  contents: string;
}

export interface AppResult {
  /** One module per `.weft` file, entries first; empty on errors. */
  logic: LogicModule[];
  /** Empty on errors and for `web`, whose DOM code lives in the logic modules. */
  views: OutputFile[];
  /** Rendered diagnostics; empty when there are none. */
  diagnostics: string;
  hasErrors: boolean;
  /** Normalized paths of every loaded source. */
  files: string[];
}

/**
 * Compiles `entries` and their imports as one app-level program, so names must be unique across the app.
 * Throws for platforms without a backend yet. Copies the result before releasing its WASM allocation.
 */
export async function compileApp(
  entries: string[],
  platform: Platform,
  load: (path: string) => string,
): Promise<AppResult> {
  const module = await loadCompiler();
  const output = module.compileApp(entries, platform, load);
  try {
    const code = output.logicCode;
    const dts = output.logicDts;
    const contents = output.viewContents;
    return {
      logic: output.logicSources.map((source, index) => ({ source, code: code[index]!, dts: dts[index]! })),
      views: output.viewPaths.map((path, index) => ({ path, contents: contents[index]! })),
      diagnostics: output.diagnostics,
      hasErrors: output.hasErrors,
      files: output.files,
    };
  } finally {
    output.free();
  }
}
