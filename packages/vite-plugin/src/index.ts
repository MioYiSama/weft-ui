import { readFileSync, readdirSync, existsSync, writeFileSync } from "node:fs";
import { dirname, join, basename } from "node:path";
import type { Plugin } from "vite";
import { compileSource } from "./compiler.ts";

/** Options for compiling Weft views and generating adjacent TypeScript declarations. */
export interface WeftOptions {
  runtime?: string;
  dts?: boolean;
}

function writeDeclarations(path: string, declarations: string) {
  const output = join(dirname(path), `${basename(path, ".weft")}.d.weft.ts`);
  if (!existsSync(output) || readFileSync(output, "utf8") !== declarations) {
    writeFileSync(output, declarations);
  }
}

function* weftFiles(directory: string): Generator<string> {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.name.startsWith(".") || entry.name === "node_modules" || entry.name === "dist") continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) yield* weftFiles(path);
    else if (entry.isFile() && entry.name.endsWith(".weft")) yield path.replaceAll("\\", "/");
  }
}

/** Compiles `.weft` imports with the Rust compiler and Web DOM backend. */
export default function weft(options: WeftOptions = {}): Plugin {
  const runtime = options.runtime ?? "@weft-ui/runtime";
  let root = process.cwd();

  return {
    name: "weft",
    enforce: "pre",
    configResolved(config) {
      root = config.root;
    },
    async buildStart() {
      if (options.dts === false) return;
      for (const entry of weftFiles(root)) {
        const output = await compileSource(entry, (path) => readFileSync(path, "utf8"), runtime);
        if (!output.hasErrors && output.dts !== undefined) writeDeclarations(entry, output.dts);
      }
    },
    async transform(code, id) {
      const entry = id.split("?", 1)[0]!.replaceAll("\\", "/");
      if (!entry.endsWith(".weft")) return;
      const output = await compileSource(entry, (path) => path === entry ? code : readFileSync(path, "utf8"), runtime);
      for (const file of output.files) this.addWatchFile(file);
      if (output.hasErrors) this.error(output.diagnostics);
      if (output.diagnostics) this.warn(output.diagnostics);
      if (options.dts !== false && output.dts !== undefined) writeDeclarations(entry, output.dts);
      return { code: output.code!, map: null };
    },
  };
}
