import { join } from "node:path";
import { fileURLToPath } from "node:url";

const DEFAULT_ROOT = fileURLToPath(new URL("../../../native/", import.meta.url));

export function nativeShell(name: "apple" | "android" | "windows" | "linux", root = DEFAULT_ROOT): string {
  return join(root, name);
}
