import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { VERSION, type NativePlatform } from "@weft-ui/runtime/protocol";

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

export function checkShellProtocol(platform: NativePlatform, shell: string): void {
  const manifest = JSON.parse(readFileSync(join(shell, "package.json"), "utf8")) as {
    name?: string;
    weft?: { protocol?: unknown };
  };
  const protocol = manifest.weft?.protocol;
  if (protocol !== VERSION) {
    throw new Error(
      `weft: protocol mismatch for ${platform}: ${manifest.name ?? nativeShellPackage(platform)} uses ` +
        `${JSON.stringify(protocol) ?? "no protocol version"}, but @weft-ui/runtime uses ${VERSION}. ` +
        "Install matching Weft UI package versions.",
    );
  }
}
