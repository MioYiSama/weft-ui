import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { type AssetFile, type Assets, hasAssets } from "./scan.ts";

/** Copies content and writes the index the WinUI shell uses for asset names and font faces. */
export function writeWindowsAssets(assets: Assets, generated: string): void {
  if (!hasAssets(assets)) return;
  const directory = join(generated, "assets");
  mkdirSync(directory, { recursive: true });
  const copy = (file: AssetFile, target: string) => {
    copyFileSync(file.path, join(directory, target));
    return target;
  };
  const images: Record<string, { File: string; Scale: number }[]> = Object.create(null);
  for (const [name, variants] of assets.images) {
    images[name] = variants.map((file) => ({ File: copy(file, `${name}.scale-${file.scale * 100}${file.ext}`), Scale: file.scale }));
  }
  for (const [name, file] of assets.icons) images[name] = [{ File: copy(file, file.file), Scale: 1 }];
  if (assets.fonts.length) mkdirSync(join(directory, "fonts"));
  const fonts = assets.fonts.map((font) => {
    copy(font, `fonts/${font.file}`);
    return { File: font.file, Family: font.family, Weight: font.weight, Italic: font.italic };
  });
  writeFileSync(join(directory, "index.json"), JSON.stringify({ Images: images, Fonts: fonts }));
}
