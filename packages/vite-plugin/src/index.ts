import { readFileSync, readdirSync, existsSync, writeFileSync } from "node:fs";
import { dirname, join, basename } from "node:path";
import type { Plugin } from "vite";
import { compileApp } from "@weft-ui/compiler";

/** Options for compiling Weft views and generating adjacent TypeScript declarations. */
export interface WeftOptions {
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
        const output = await compileApp([entry], "web", (path) => readFileSync(path, "utf8"));
        const logic = output.logic.find((module) => module.source === entry);
        if (!output.hasErrors && logic) writeDeclarations(entry, logic.dts);
      }
    },
    async transform(code, id) {
      const entry = id.split("?", 1)[0]!.replaceAll("\\", "/");
      if (!entry.endsWith(".weft")) return;
      const output = await compileApp([entry], "web", (path) => path === entry ? code : readFileSync(path, "utf8"));
      for (const file of output.files) this.addWatchFile(file);
      if (output.hasErrors) this.error(output.diagnostics);
      if (output.diagnostics) this.warn(output.diagnostics);
      const logic = output.logic.find((module) => module.source === entry)!;
      if (options.dts !== false) writeDeclarations(entry, logic.dts);
      return { code: logic.code, map: null };
    },
  };
}
