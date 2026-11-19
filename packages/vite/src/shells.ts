import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import type { NativePlatform } from "@weft-ui/runtime/protocol";

const require = createRequire(import.meta.url);

const TARGETS: Record<NativePlatform, string> = {
  ios: "apple",
  macos: "apple",
  android: "android",
  windows: "windows",
  linux: "linux",
};

export function nativeShellPackage(platform: NativePlatform): string {
  return `@weft-ui/native-${TARGETS[platform]}`;
}

export function nativeShell(platform: NativePlatform, root?: string): string {
  if (root !== undefined) return join(root, TARGETS[platform]);
  const name = nativeShellPackage(platform);
  try {
    return dirname(require.resolve(`${name}/package.json`));
  } catch (cause) {
    if ((cause as NodeJS.ErrnoException).code !== "MODULE_NOT_FOUND") throw cause;
    throw new Error(`weft: building for ${platform} needs the native shell; install it with \`pnpm add -D ${name}\``);
  }
}
