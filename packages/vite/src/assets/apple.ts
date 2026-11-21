import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Assets } from "./scan.ts";

const info = { author: "weft", version: 1 };

/** Materializes the catalog and font resources of the generated Swift package. */
export function writeAppleAssets({ images, icons, fonts }: Assets, generated: string): void {
  if (images.size || icons.size) {
    const catalog = join(generated, "Assets.xcassets");
    mkdirSync(catalog, { recursive: true });
    const contents = (directory: string, value: object) =>
      writeFileSync(join(directory, "Contents.json"), JSON.stringify({ ...value, info }, null, 2) + "\n");
    contents(catalog, {});
    for (const [name, variants] of images) {
      const set = join(catalog, `${name}.imageset`);
      mkdirSync(set);
      const entries = variants.map(({ path, file, scale }) => {
        copyFileSync(path, join(set, file));
        return { idiom: "universal", filename: file, scale: `${scale}x` };
      });
      contents(set, { images: entries });
    }
    for (const [name, { path, file }] of icons) {
      const set = join(catalog, `${name}.imageset`);
      mkdirSync(set);
      copyFileSync(path, join(set, file));
      contents(set, {
        images: [{ idiom: "universal", filename: file }],
        properties: { "preserves-vector-representation": true },
      });
    }
  }
  if (fonts.length) {
    const directory = join(generated, "fonts");
    mkdirSync(directory, { recursive: true });
    const entries = fonts.map(({ path, file, family, weight, italic }) => {
      copyFileSync(path, join(directory, file));
      return { file, family, weight, italic };
    });
    writeFileSync(join(directory, "manifest.json"), JSON.stringify(entries, null, 2) + "\n");
  }
}
