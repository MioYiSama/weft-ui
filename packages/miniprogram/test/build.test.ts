import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, beforeEach, expect, test } from "vitest";
import { build } from "../src/index.ts";

const fixtures = fileURLToPath(new URL("./fixtures/", import.meta.url));
const entry = join(fixtures, "app/Main.weft");
let temporary: string;
let outDir: string;
let runtimeDir: string;

beforeEach(async () => {
  temporary = await mkdtemp(join(tmpdir(), "weft-mp-"));
  outDir = join(temporary, "output");
  runtimeDir = join(temporary, "runtime");
  await mkdir(runtimeDir);
  await writeFile(join(runtimeDir, "remote.js"), 'export * from "./core.js";\n');
  await writeFile(join(runtimeDir, "wechat.js"), "export const view = () => ({});\n");
  await writeFile(join(runtimeDir, "core.js"), "export {};\n");
  await writeFile(join(runtimeDir, "ignored.d.ts"), "export {};\n");
});

afterEach(async () => {
  await rm(temporary, { recursive: true, force: true });
});

test("builds the import closure, view component and runtime with resolvable JS imports", async () => {
  const result = await build({ entries: [entry], outDir, runtimeDir });
  expect(result.warnings).toBe("");
  expect(result.written.toSorted()).toEqual([
    "Main-view.js", "Main-view.json", "Main-view.wxml", "Main.d.ts", "Main.js",
    "shared/models.d.ts", "shared/models.js", "weft-layout.js",
    "weft-runtime/core.js", "weft-runtime/remote.js", "weft-runtime/wechat.js",
  ].toSorted());
  expect(JSON.parse(await readFile(join(outDir, "Main-view.json"), "utf8"))).toEqual({ component: true });
  expect(await readFile(join(outDir, "Main-view.wxml"), "utf8")).toContain("wx:for");

  let imports = 0;
  for (const path of result.written.filter((path) => path.endsWith(".js"))) {
    const source = await readFile(join(outDir, path), "utf8");
    for (const match of source.matchAll(/(?:from\s*|import\s*)["'](\.[^"']+)["']/g)) {
      const target = relative(outDir, resolve(outDir, dirname(path), match[1]!)).replaceAll("\\", "/");
      expect(result.written, `${path} imports ${match[1]}`).toContain(target);
      imports++;
    }
  }
  expect(imports).toBeGreaterThan(0);
});

test("does not rewrite unchanged files and reports only changed runtime files", async () => {
  const options = { entries: [entry], outDir, runtimeDir };
  await build(options);
  expect((await build(options)).written).toEqual([]);
  await writeFile(join(runtimeDir, "core.js"), "export const changed = true;\n");
  expect((await build(options)).written).toEqual(["weft-runtime/core.js"]);
  expect(await readFile(join(outDir, "weft-runtime/core.js"), "utf8"))
    .toBe("export const changed = true;\n");
});

test("preserves paths under an explicit root and merges multiple entry closures", async () => {
  const result = await build({
    entries: [entry, join(fixtures, "app/Secondary.weft")], root: fixtures, outDir, runtimeDir,
  });
  expect(result.written).toEqual(expect.arrayContaining([
    "app/Main.js", "app/Main-view.js", "app/Secondary.js", "app/Secondary-view.js", "app/shared/models.js",
  ]));
  expect(result.written.filter((path) => path === "app/shared/models.js")).toHaveLength(1);
  expect(await readFile(join(outDir, "app/Main.js"), "utf8")).toContain("../weft-runtime/remote.js");
});

test("rejects entries outside the source root", async () => {
  await expect(build({ entries: [entry], root: join(fixtures, "escape"), outDir, runtimeDir }))
    .rejects.toThrow(/outside source root/);
});

test("renders loader diagnostics when an import escapes the source root", async () => {
  await expect(build({ entries: [join(fixtures, "escape/Main.weft")], outDir, runtimeDir }))
    .rejects.toThrow(/cannot load module.*outside source root/s);
});

test("rejects absolute module imports with a root diagnostic", async () => {
  const absoluteEntry = join(temporary, "Absolute.weft");
  await writeFile(absoluteEntry, `import ${JSON.stringify(join(fixtures, "x.weft"))}\n`);
  await expect(build({ entries: [absoluteEntry], outDir, runtimeDir }))
    .rejects.toThrow(/outside source root/);
});

test("throws rendered type diagnostics with a source location", async () => {
  await expect(build({ entries: [join(fixtures, "TypeError.weft")], outDir, runtimeDir }))
    .rejects.toThrow(/mismatched types: expected `String`, found `Int`[\s\S]*TypeError\.weft:2:/);
});

test("requires an entry", async () => {
  await expect(build({ entries: [], outDir, runtimeDir })).rejects.toThrow(/entry is required/);
});
