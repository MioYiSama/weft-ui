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

/** Internal compilation boundary; copies the result before releasing its WASM allocation. */
export async function compileSource(entry: string, load: (path: string) => string, runtime: string) {
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
