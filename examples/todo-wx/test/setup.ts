import { build } from "@weft-ui/miniprogram";
import { fileURLToPath } from "node:url";

export default async function setup() {
  const root = fileURLToPath(new URL("../", import.meta.url));
  await build({ entries: [`${root}ui/TodoList.weft`], outDir: `${root}miniprogram/weft` });
}
