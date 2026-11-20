import { execFileSync, type ExecFileSyncOptions } from "node:child_process";
import { mkdirSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";
import { pages, type Platform } from "../pages.ts";

export const settleMs = 1000;

export interface Shot {
  id: string;
  scheme: "light" | "dark";
  name: string;
  path: string;
}

export function prepareShots(platform: Platform): { directory: string; shots: Shot[] } {
  const directory = resolve("screenshots", "actual", platform);
  mkdirSync(directory, { recursive: true });
  const shots: Shot[] = [];
  for (const { id, dark } of pages(platform)) {
    shots.push({ id, scheme: "light", name: `${id}.png`, path: join(directory, `${id}.png`) });
    if (dark) {
      shots.push({ id, scheme: "dark", name: `${id}-dark.png`, path: join(directory, `${id}-dark.png`) });
    }
  }
  for (const { path } of shots) rmSync(path, { force: true });
  return { directory, shots };
}

export function command(executable: string, args: string[], options: ExecFileSyncOptions = {}): void {
  execFileSync(executable, args, { stdio: "inherit", ...options });
}

export function build(platform: Platform, env = process.env): void {
  const pnpm = process.env.npm_execpath;
  if (!pnpm) throw new Error(`Run this driver with pnpm --filter showcase shot:${platform}`);
  command(process.execPath, [pnpm, "run", `build:${platform}`], { env });
}
