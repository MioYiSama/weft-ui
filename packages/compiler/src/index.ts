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

/** Compiles Web bindings, copying the result before releasing its WASM allocation. */
export async function compileWeb(entry: string, load: (path: string) => string, runtime: string) {
  const module = await loadCompiler();
  const output = module.compile(entry, load, runtime);
  try {
    return {
      code: output.code,
      dts: output.dts,
      diagnostics: output.diagnostics,
      hasErrors: output.hasErrors,
      files: output.files,
    };
  } finally {
    output.free();
  }
}

/** Generated file paths are relative to the output directory. */
export interface MiniprogramResult {
  files?: { path: string; contents: string }[];
  diagnostics: string;
  hasErrors: boolean;
  sources: string[];
}

/** Compiles mini program files; entry and loader paths must be relative to a source root. */
export async function compileMiniprogram(
  entry: string,
  load: (path: string) => string,
  runtime: string,
): Promise<MiniprogramResult> {
  const module = await loadCompiler();
  const output = module.compileMiniprogram(entry, load, runtime);
  try {
    const paths = output.paths;
    const contents = output.contents;
    return {
      files: paths?.map((path, index) => ({ path, contents: contents![index]! })),
      diagnostics: output.diagnostics,
      hasErrors: output.hasErrors,
      sources: output.files,
    };
  } finally {
    output.free();
  }
}
