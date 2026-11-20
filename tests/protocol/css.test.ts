import { readFileSync } from "node:fs";
import { expect, test } from "vitest";
import { declarations, type Css } from "../../packages/runtime/src/css.ts";
import type { Value } from "../../packages/runtime/src/remote.ts";

const { cases } = JSON.parse(readFileSync(new URL("../../protocol/corpus/css.json", import.meta.url), "utf8")) as {
  cases: { css: Css; value: Value; out: string }[];
};

test.each(cases)("CSS declaration $css for $value", ({ css, value, out }) => {
  expect(declarations(css, value)).toBe(out);
});

test("resolves tokens against the active app theme", () => {
  const css = { unit: "color", property: "background-color" } as const;
  const tokens = new Set(["surface"]);
  expect(declarations(css, "@surface", tokens)).toBe("background-color:var(--weft-surface)");
  expect(declarations(css, "@absent", tokens)).toBe("background-color:transparent");
  expect(declarations(css, "@surface", null)).toBe("background-color:transparent");
});
