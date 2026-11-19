import { cpSync, rmSync } from "node:fs";

const [phase, target] = process.argv.slice(2);
if (!["prepack", "postpack"].includes(phase) || !["apple", "android", "windows", "linux"].includes(target)) {
  console.error("usage: node native/pack.mjs <prepack|postpack> <apple|android|windows|linux>");
  process.exit(1);
}

const shell = new URL(`${target}/`, import.meta.url);
if (target === "linux") {
  const copy = new URL("core/", shell);
  if (phase === "prepack") {
    cpSync(new URL("core/include/", import.meta.url), new URL("include/", copy), { recursive: true });
    cpSync(new URL("core/src/ops.cpp", import.meta.url), new URL("src/ops.cpp", copy));
  } else {
    rmSync(copy, { recursive: true, force: true });
  }
}
