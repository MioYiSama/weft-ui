import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const builds = [
  { config: "tsconfig.wx.json", committed: "../../miniprogram/runtime/", files: ["core.js", "wx.js"] },
];

test.each(builds)("committed $config output matches a fresh TypeScript build", async ({ config, committed, files }) => {
  const output = await mkdtemp(join(tmpdir(), "weft-runtime-"));
  try {
    const runtime = fileURLToPath(new URL("../", import.meta.url));
    const committedDir = fileURLToPath(new URL(committed, import.meta.url));
    const typescript = createRequire(import.meta.url).resolve("typescript/package.json");
    const tsc = join(dirname(typescript), "bin/tsc");
    execFileSync(process.execPath, [tsc, "-p", config, "--outDir", output], {
      cwd: runtime,
      stdio: "pipe",
    });
    expect((await readdir(output)).sort()).toEqual(files);
    expect((await readdir(committedDir)).filter((name) => name.endsWith(".js")).sort()).toEqual(files);
    for (const file of files) {
      expect(await readFile(join(committedDir, file), "utf8"), `run pnpm build:runtime (${file})`)
        .toBe(await readFile(join(output, file), "utf8"));
    }
  } finally {
    await rm(output, { recursive: true, force: true });
  }
}, 15_000);
