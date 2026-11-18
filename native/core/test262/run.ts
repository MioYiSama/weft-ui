/**
 * Runs a test262 subset through `hermesc` and the Hermes VM to find the
 * syntax `@weft-ui/vite` must lower for native platforms (`HERMES_TARGET`
 * in packages/vite/src/native.ts). Every syntax feature that Oxc can
 * lower is probed with the language tests tagged with it; the result is a
 * Markdown table of compile and run results per feature.
 *
 *   node native/core/test262/run.ts --test262 <checkout> --hermesc <path> --hvm <path> [--limit 300] [-- <hermesc flags>]
 *
 * `hvm` is built by `cmake --build native/core/build --target hvm`.
 */

import { execFile } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { availableParallelism, tmpdir } from "node:os";
import { join, relative } from "node:path";
import { parseArgs, promisify } from "node:util";

const run = promisify(execFile);

/** Probed features by the edition that introduced them; Oxc lowers each of them. */
const FEATURES: [edition: string, feature: string][] = [
  ["ES2015", "let"],
  ["ES2015", "class"],
  ["ES2015", "generators"],
  ["ES2016", "exponentiation"],
  ["ES2017", "async-functions"],
  ["ES2018", "object-rest"],
  ["ES2018", "object-spread"],
  ["ES2018", "async-iteration"],
  ["ES2018", "regexp-named-groups"],
  ["ES2018", "regexp-lookbehind"],
  ["ES2018", "regexp-dotall"],
  ["ES2019", "optional-catch-binding"],
  ["ES2020", "optional-chaining"],
  ["ES2020", "coalesce-expression"],
  ["ES2021", "logical-assignment-operators"],
  ["ES2021", "numeric-separator-literal"],
  ["ES2022", "class-fields-public"],
  ["ES2022", "class-fields-private"],
  ["ES2022", "class-methods-private"],
  ["ES2022", "class-static-fields-public"],
  ["ES2022", "class-static-fields-private"],
  ["ES2022", "class-static-methods-private"],
  ["ES2022", "class-static-block"],
  ["ES2022", "class-fields-private-in"],
  ["ES2022", "regexp-match-indices"],
  ["ES2024", "regexp-v-flag"],
  ["ES2025", "regexp-modifiers"],
  ["ES2025", "regexp-duplicate-named-groups"],
];

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    test262: { type: "string" },
    hermesc: { type: "string" },
    hvm: { type: "string" },
    limit: { type: "string", default: "300" },
  },
});
if (!values.test262 || !values.hermesc || !values.hvm) {
  throw new Error("usage: run.ts --test262 <checkout> --hermesc <path> --hvm <path> [--limit N] [-- <hermesc flags>]");
}
const test262 = values.test262;
const hermesc = values.hermesc;
const hvm = values.hvm;
const limit = Number(values.limit);
const flags = positionals;

interface Meta {
  features: string[];
  includes: string[];
  flags: string[];
  negative: { phase: string; type: string } | null;
}

/** Reads the YAML front matter fields the runner needs. */
function meta(source: string): Meta {
  const yaml = /\/\*---([\s\S]*?)---\*\//.exec(source)?.[1] ?? "";
  const list = (key: string): string[] => {
    const inline = new RegExp(`^${key}:\\s*\\[([^\\]]*)\\]`, "m").exec(yaml);
    if (inline) return inline[1]!.split(",").map((s) => s.trim()).filter(Boolean);
    const block = new RegExp(`^${key}:\\s*\\n((?:\\s+-.*\\n?)+)`, "m").exec(yaml);
    return block ? block[1]!.split("\n").map((l) => l.replace(/^\s*-\s*/, "").trim()).filter(Boolean) : [];
  };
  const negative = /^negative:\s*\n\s+phase:\s*(\S+)\s*\n\s+type:\s*(\S+)/m.exec(yaml);
  return {
    features: list("features"),
    includes: list("includes"),
    flags: list("flags"),
    negative: negative ? { phase: negative[1]!, type: negative[2]! } : null,
  };
}

function* walk(directory: string): Generator<string> {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) yield* walk(path);
    else if (entry.name.endsWith(".js") && !entry.name.includes("_FIXTURE")) yield path;
  }
}

interface Test {
  path: string;
  source: string;
  meta: Meta;
}

const byFeature = new Map<string, Test[]>(FEATURES.map(([, feature]) => [feature, []]));
for (const root of ["test/language", "test/built-ins/RegExp"]) {
  for (const path of walk(join(test262, root))) {
    const source = readFileSync(path, "utf8");
    const info = meta(source);
    // Modules and raw tests need a different harness; the bundle is a script.
    if (info.flags.includes("module") || info.flags.includes("raw")) continue;
    for (const feature of info.features) byFeature.get(feature)?.push({ path, source, meta: info });
  }
}

const harness = new Map<string, string>();
const include = (name: string) => {
  let text = harness.get(name);
  if (text === undefined) {
    text = readFileSync(join(test262, "harness", name), "utf8");
    harness.set(name, text);
  }
  return text;
};

type Outcome = "pass" | "compile" | "run";

const scratch = mkdtempSync(join(tmpdir(), "weft-test262-"));
let counter = 0;

async function execute(test: Test): Promise<Outcome> {
  const { meta: info } = test;
  const async = info.flags.includes("async");
  const strict = info.flags.includes("onlyStrict");
  const parts = [...(strict ? ['"use strict";'] : []), include("assert.js"), include("sta.js")];
  if (async) parts.push(include("doneprintHandle.js"));
  for (const name of info.includes) parts.push(include(name));
  parts.push(test.source);
  const id = counter++;
  const js = join(scratch, `${id}.js`);
  const hbc = join(scratch, `${id}.hbc`);
  writeFileSync(js, parts.join("\n"));
  const early = info.negative?.phase === "parse" || info.negative?.phase === "early";
  try {
    await run(hermesc, ["-emit-binary", "-w", ...flags, `-out=${hbc}`, js]);
  } catch {
    return early ? "pass" : "compile";
  }
  if (early) return "compile";
  try {
    const { stdout } = await run(hvm, [hbc], { timeout: 10_000 });
    if (info.negative) return "run";
    if (async && !stdout.includes("Test262:AsyncTestComplete")) return "run";
    return "pass";
  } catch {
    return info.negative ? "pass" : "run";
  }
}

/** Runs `tasks` with bounded concurrency. */
async function pool<T, R>(items: T[], task: (item: T) => Promise<R>): Promise<R[]> {
  const results = new Array<R>(items.length);
  let next = 0;
  const worker = async () => {
    while (next < items.length) {
      const index = next++;
      results[index] = await task(items[index]!);
    }
  };
  await Promise.all(Array.from({ length: availableParallelism() }, worker));
  return results;
}

/** Evenly spaced sample of at most `limit` tests, stable across runs. */
function sample(tests: Test[]): Test[] {
  if (tests.length <= limit) return tests;
  const step = tests.length / limit;
  return Array.from({ length: limit }, (_, i) => tests[Math.floor(i * step)]!);
}

const rows = ["| Edition | Feature | Tests | Pass | Compile errors | Runtime failures |", "| --- | --- | --- | --- | --- | --- |"];
const failures: string[] = [];
try {
  for (const [edition, feature] of FEATURES) {
    const tests = sample(byFeature.get(feature)!);
    const outcomes = await pool(tests, execute);
    const count = (outcome: Outcome) => outcomes.filter((o) => o === outcome).length;
    rows.push(`| ${edition} | \`${feature}\` | ${tests.length} | ${count("pass")} | ${count("compile")} | ${count("run")} |`);
    outcomes.forEach((outcome, i) => {
      if (outcome !== "pass") failures.push(`${feature}\t${outcome}\t${relative(test262, tests[i]!.path)}`);
    });
  }
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
console.log(`hermesc flags: ${flags.length ? flags.join(" ") : "(none)"}\n`);
console.log(rows.join("\n"));
if (process.env.WEFT_TEST262_FAILURES) writeFileSync(process.env.WEFT_TEST262_FAILURES, failures.join("\n") + "\n");
