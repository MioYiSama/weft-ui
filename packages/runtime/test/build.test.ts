import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

test("embedded runtime matches a fresh TypeScript build", async () => {
  const output = await mkdtemp(join(tmpdir(), "weft-runtime-"));
  try {
    const runtime = fileURLToPath(new URL("../", import.meta.url));
    const committed = fileURLToPath(new URL("../../../src/js/", import.meta.url));
    const typescript = createRequire(import.meta.url).resolve("typescript/package.json");
    const tsc = join(dirname(typescript), "bin/tsc");
    execFileSync(process.execPath, [tsc, "-p", "tsconfig.rust.json", "--outDir", output], {
      cwd: runtime,
      stdio: "pipe",
    });
    const files = (await readdir(output)).sort();
    expect(files).toEqual(["index.js", "remote.js"]);
    expect((await readdir(committed)).filter((name) => name.endsWith(".js")).sort()).toEqual(files);
    for (const file of files) {
      expect(await readFile(join(committed, file), "utf8"), `run pnpm build:runtime (${file})`)
        .toBe(await readFile(join(output, file), "utf8"));
    }
  } finally {
    await rm(output, { recursive: true, force: true });
  }
}, 15_000);
