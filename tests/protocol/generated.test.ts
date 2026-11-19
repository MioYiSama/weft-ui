import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

test("generated protocol files match the schema", () => {
  const generator = fileURLToPath(new URL("../../protocol/generate.ts", import.meta.url));
  const result = spawnSync(process.execPath, [generator, "--check"], { encoding: "utf8" });
  expect(result.status, result.error?.message ?? result.stderr).toBe(0);
});
