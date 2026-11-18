// @vitest-environment node
import { execFile } from "node:child_process";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify, stripVTControlCharacters } from "node:util";
import { expect, test } from "vitest";

const fixture = fileURLToPath(new URL("./fixture", import.meta.url));
const tsc = join(dirname(createRequire(import.meta.url).resolve("typescript/package.json")), "bin/tsc");

/** Runs `tsc` with the Weft content mapper on `project` and returns its report; pretty output includes related locations. */
async function check(project: string): Promise<string> {
  try {
    await promisify(execFile)(process.execPath, [tsc, "-p", project, "--runExternalCode", "--pretty"], { cwd: fixture });
    return "";
  } catch (error) {
    if (error instanceof Error && "stdout" in error && typeof error.stdout === "string") {
      return stripVTControlCharacters(error.stdout);
    }
    throw error;
  }
}

// The fixtures start with non-ASCII comments, so positions only line up when
// the mapper and TypeScript agree on the position encoding.
test("TypeScript checks .weft imports and points into their declarations", async () => {
  expect(await check("tsconfig.json")).toBe(`index.ts:7:29 - error TS2322: Type 'string' is not assignable to type 'number'.

7 export const todo: Todo = { id: "1", name: "first" };
                              ~~

  models.weft:3:3 - The expected type comes from property 'id' which is declared here on type 'Todo'
    3   id: Int,
        ~~


Found 1 error in index.ts:7

`);
});

test("Weft errors are reported in the .weft file", async () => {
  expect(await check("tsconfig.broken.json")).toBe(`Broken.weft:3:10 - error weft1: unknown type \`Missing\`

3   value: Missing,
           ~~~~~~~


Found 1 error in Broken.weft:3

`);
});
