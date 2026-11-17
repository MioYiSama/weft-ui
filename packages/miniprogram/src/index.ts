import { readFileSync } from "node:fs";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, isAbsolute, join, posix, relative, resolve, win32 } from "node:path";
import { fileURLToPath } from "node:url";
import { compileMiniprogram } from "@weft-ui/compiler";

/** Sources share a root; generated paths preserve their layout beneath outDir. */
export interface BuildOptions {
  entries: string[];
  outDir: string;
  /** Defaults to the first entry's directory. Entry paths are relative to the working directory. */
  root?: string;
  /** @internal Overrides the bundled runtime directory for tests. */
  runtimeDir?: string;
}

/** Only files whose contents changed appear in written, relative to outDir. */
export interface BuildResult {
  written: string[];
  warnings: string;
}

function insideRoot(path: string): boolean {
  return !isAbsolute(path) && !win32.isAbsolute(path) && path !== ".." && !path.startsWith("../");
}

async function writeChanged(path: string, contents: string): Promise<boolean> {
  try {
    if (await readFile(path, "utf8") === contents) return false;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, contents);
  return true;
}

/** Compiles entry closures and copies the runtime, throwing rendered diagnostics on errors. */
export async function build(options: BuildOptions): Promise<BuildResult> {
  if (options.entries.length === 0) throw new Error("At least one Weft entry is required.");
  const root = resolve(options.root ?? dirname(resolve(options.entries[0]!)));
  const entries = options.entries.map((entry) => {
    const path = relative(root, resolve(entry)).replaceAll("\\", "/");
    if (!insideRoot(path)) throw new Error(`Entry ${entry} is outside source root ${root}.`);
    return path;
  });
  const load = (path: string): string => {
    const normalized = posix.normalize(path.replaceAll("\\", "/"));
    if (!insideRoot(normalized)) {
      throw new Error(`Module ${path} is outside source root ${root}.`);
    }
    return readFileSync(join(root, normalized), "utf8");
  };
  const files = new Map<string, string>();
  const warnings: string[] = [];
  for (const entry of entries) {
    const output = await compileMiniprogram(entry, load, "weft-runtime/wx.js");
    if (output.hasErrors) throw new Error(output.diagnostics);
    if (output.diagnostics) warnings.push(output.diagnostics);
    for (const file of output.files!) files.set(file.path, file.contents);
  }

  const runtimeDir = options.runtimeDir ?? fileURLToPath(new URL("../runtime/", import.meta.url));
  for (const file of await readdir(runtimeDir, { withFileTypes: true })) {
    if (file.isFile() && file.name.endsWith(".js")) {
      files.set(`weft-runtime/${file.name}`, await readFile(join(runtimeDir, file.name), "utf8"));
    }
  }

  const written: string[] = [];
  for (const [path, contents] of files) {
    if (await writeChanged(join(options.outDir, path), contents)) written.push(path);
  }
  return { written, warnings: warnings.join("\n") };
}
