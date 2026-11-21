import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import convert from "svg2vectordrawable";
import type { Assets, Font } from "./scan.ts";

const densities = ["mdpi", "xhdpi", "xxhdpi"] as const;

/** Generates Android resources; aapt groups image variants by name and density. */
export async function writeAndroidAssets({ images, icons, fonts }: Assets, generated: string): Promise<void> {
  const res = join(generated, "src/main/res");
  for (const [name, variants] of images) {
    for (const file of variants) {
      const directory = join(res, `drawable-${densities[file.scale - 1]}`);
      mkdirSync(directory, { recursive: true });
      copyFileSync(file.path, join(directory, `weft_${name}${file.ext}`));
    }
  }
  if (icons.size) {
    const directory = join(res, "drawable");
    mkdirSync(directory, { recursive: true });
    for (const [name, file] of icons) {
      try {
        const xml = await convert(readFileSync(file.path, "utf8"), { strict: true, fillBlack: true });
        writeFileSync(join(directory, `weft_${name}.xml`), `${xml.trimEnd()}\n`);
      } catch (cause) {
        throw new Error(`weft: could not convert SVG icon \`${file.file}\` to VectorDrawable: ${cause instanceof Error ? cause.message : cause}`, { cause });
      }
    }
  }
  if (fonts.length) writeFonts(fonts, join(res, "font"));
}

function writeFonts(fonts: readonly Font[], directory: string): void {
  const families = new Map<string, { family: string; entries: string[] }>();
  const faces = new Map<string, string>();
  mkdirSync(directory, { recursive: true });
  for (const font of fonts) {
    const familyName = `weft_${font.family.toLowerCase().replace(/[^a-z0-9]/gu, "_")}`;
    let family = families.get(familyName);
    if (!family) {
      family = { family: font.family, entries: [] };
      families.set(familyName, family);
    } else if (family.family !== font.family) {
      throw new Error(`weft: font families \`${font.family}\` and \`${family.family}\` share Android resource name \`${familyName}\``);
    }
    const face = `${familyName}_${font.weight}${font.italic ? "_italic" : ""}`;
    const previous = faces.get(face);
    if (previous) throw new Error(`weft: font files \`${font.file}\` and \`${previous}\` share Android resource name \`${face}\``);
    faces.set(face, font.file);
    copyFileSync(font.path, join(directory, `${face}${font.ext}`));
    family.entries.push(`    <font android:font="@font/${face}" android:fontWeight="${font.weight}" android:fontStyle="${font.italic ? "italic" : "normal"}" />`);
  }
  for (const [name, { family, entries }] of families) {
    const face = faces.get(name);
    if (face) throw new Error(`weft: font family \`${family}\` and file \`${face}\` share Android resource name \`${name}\``);
    writeFileSync(join(directory, `${name}.xml`),
      `<font-family xmlns:android="http://schemas.android.com/apk/res/android">\n${entries.join("\n")}\n</font-family>\n`);
  }
}
