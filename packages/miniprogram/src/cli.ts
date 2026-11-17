#!/usr/bin/env node
import { parseArgs } from "node:util";
import { build } from "./index.ts";

try {
  const { values, positionals } = parseArgs({
    options: {
      root: { type: "string" },
      out: { type: "string" },
    },
    allowPositionals: true,
  });
  if (!values.out || positionals.length === 0) {
    throw new Error("Usage: weft-mp [--root <dir>] --out <dir> <entry.weft>...");
  }
  const result = await build({ entries: positionals, outDir: values.out, root: values.root });
  if (result.warnings) console.error(result.warnings);
  console.log(`Weft: wrote ${result.written.length} files to ${values.out}.`);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
