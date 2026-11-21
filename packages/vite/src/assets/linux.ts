import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import type { Assets } from "./scan.ts";

/** Copies Linux media and fonts into the generated project's asset directory. */
export function writeLinuxAssets({ images, icons, fonts }: Assets, generated: string): void {
  const directory = join(generated, "assets");
  if (images.size || icons.size) {
    mkdirSync(directory, { recursive: true });
    for (const variants of images.values()) {
      for (const file of variants) copyFileSync(file.path, join(directory, file.file));
    }
    for (const file of icons.values()) copyFileSync(file.path, join(directory, file.file));
  }
  if (fonts.length) {
    const fontDirectory = join(directory, "fonts");
    mkdirSync(fontDirectory, { recursive: true });
    for (const file of fonts) copyFileSync(file.path, join(fontDirectory, file.file));
  }
}
