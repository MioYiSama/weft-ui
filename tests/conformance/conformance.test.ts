/**
 * Protocol conformance: every fixture runs its script on the headless
 * renderer, the mini program and the Web. After every step all platforms
 * must render the same neutral tree and their host handlers must have seen
 * the same events. The headless renderer's ops are compared with the
 * fixture's golden file; run with `WEFT_UPDATE_SNAPSHOTS=1` to rewrite them.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "vitest";
import { script as board } from "../fixtures/board/script.ts";
import { script as gallery } from "../fixtures/gallery/script.ts";
import { script as language } from "../fixtures/language/script.ts";
import { script as todo } from "../fixtures/todo/script.ts";
import { type AppLike, type Driver, type Script, buildFixture, headless, web, wechat } from "./harness.ts";

const fixtures: [string, Script<AppLike>][] = [
  ["todo", todo as unknown as Script<AppLike>],
  ["board", board as unknown as Script<AppLike>],
  ["gallery", gallery as unknown as Script<AppLike>],
  ["language", language as unknown as Script<AppLike>],
];

test.each(fixtures)("%s renders alike on every platform and matches its golden ops", async (name, script) => {
  const built = await buildFixture(script);
  const renderer = headless(script, built);
  const drivers: Driver<AppLike>[] = [renderer, await wechat(script, built), web(script)];
  const golden: string[] = [];
  let seen = 0;

  const check = (step: string) => {
    const [reference, ...others] = drivers;
    for (const driver of others) {
      expect(driver.tree(), `${driver.name} after "${step}"`).toEqual(reference!.tree());
      expect(driver.log, `${driver.name} events after "${step}"`).toEqual(reference!.log);
    }
    const batches = renderer.renderer.batches.slice(seen);
    seen = renderer.renderer.batches.length;
    golden.push(`# ${step}`);
    golden.push(...(batches.length === 0 ? ["(no ops)"] : batches.map((lines) => lines.join("\n"))));
    golden.push("");
  };

  try {
    for (const driver of drivers) await driver.settle();
    check("mount");
    for (const step of script.steps) {
      for (const driver of drivers) {
        step.run(driver, driver.app.model);
        await driver.settle();
      }
      check(step.name);
    }
    for (const driver of drivers) driver.app.unmount();
  } finally {
    await rm(built.out, { recursive: true, force: true });
  }

  const path = join(import.meta.dirname, `./golden/${name}.ops`);
  const text = golden.join("\n");
  if (process.env.WEFT_UPDATE_SNAPSHOTS) writeFileSync(path, text);
  else expect(text, `golden/${name}.ops (run with WEFT_UPDATE_SNAPSHOTS=1 to update)`).toBe(readFileSync(path, "utf8"));
});
