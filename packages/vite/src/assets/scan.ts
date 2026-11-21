import { readFileSync, readdirSync } from "node:fs";
import { extname, join } from "node:path";
import { fontFamily, type FontWeight } from "./font.ts";

export interface AssetFile {
  /** Absolute source path, independent of the target platform. */
  readonly path: string;
  /** Original basename, including any density suffix and file extension. */
  readonly file: string;
  readonly ext: string;
}

export interface ImageVariant extends AssetFile {
  readonly scale: 1 | 2 | 3;
}

export interface Font extends AssetFile {
  readonly family: string;
  readonly weight: FontWeight;
  readonly italic: boolean;
}

export interface Assets {
  /** Indexed by logical name, with variants in ascending density order. */
  readonly images: ReadonlyMap<string, readonly ImageVariant[]>;
  readonly icons: ReadonlyMap<string, AssetFile>;
  readonly fonts: readonly Font[];
}

const imageExts = [".png", ".jpg", ".jpeg", ".webp"];
const fontExts = [".ttf", ".otf"];
const iconExts = [".svg"];
const weights = { Light: 300, Regular: 400, Medium: 500, SemiBold: 600, Bold: 700 } as const;

function* files(directory: string, extensions: readonly string[]): Generator<AssetFile> {
  let entries;
  try {
    entries = readdirSync(directory, { withFileTypes: true });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    return;
  }
  for (const entry of entries.sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
    if (!entry.isFile() && !entry.isSymbolicLink()) continue;
    const { name: file } = entry;
    const ext = extname(file);
    if (extensions.includes(ext)) yield { file, path: join(directory, file), ext };
  }
}

function checkName(name: string): void {
  if (!/^[a-z0-9_]+$/.test(name)) {
    throw new Error(`weft: asset name \`${name}\` may only contain a-z, 0-9 and _`);
  }
}

/** Discovers, validates, and groups local assets for the platform emitters. */
export function scanAssets(directory: string): Assets {
  const images = new Map<string, ImageVariant[]>();
  for (const file of files(join(directory, "images"), imageExts)) {
    const base = file.file.slice(0, -file.ext.length);
    const suffix = /@([23])x$/.exec(base);
    const name = suffix ? base.slice(0, -suffix[0].length) : base;
    checkName(name);
    const scale = suffix?.[1] === "2" ? 2 : suffix?.[1] === "3" ? 3 : 1;
    const variant = { ...file, scale } as const;
    const variants = images.get(name);
    if (!variants) images.set(name, [variant]);
    else {
      if (variants.some((item) => item.scale === scale)) {
        throw new Error(`weft: asset \`${name}\` has more than one ${scale}x image`);
      }
      variants.push(variant);
      variants.sort((a, b) => a.scale - b.scale);
    }
  }

  const icons = new Map<string, AssetFile>();
  for (const file of files(join(directory, "icons"), iconExts)) {
    const name = file.file.slice(0, -file.ext.length);
    checkName(name);
    if (images.has(name)) throw new Error(`weft: asset \`${name}\` is both an image and an icon`);
    icons.set(name, file);
  }

  const fonts: Font[] = [];
  for (const file of files(join(directory, "fonts"), fontExts)) {
    const match = /^.+-(Light|Regular|Medium|SemiBold|Bold)(Italic)?\.(ttf|otf)$/.exec(file.file);
    if (!match) {
      throw new Error(`weft: font file \`${file.file}\` must end in -<Weight>[Italic].ttf or -<Weight>[Italic].otf (Light, Regular, Medium, SemiBold, Bold)`);
    }
    const weight = weights[match[1] as keyof typeof weights];
    const italic = match[2] !== undefined;
    try {
      fonts.push({ ...file, family: fontFamily(readFileSync(file.path)), weight, italic });
    } catch (cause) {
      throw new Error(`weft: could not read family from font \`${file.file}\`: ${(cause as Error).message}`, { cause });
    }
  }
  return { images, icons, fonts };
}

/** Whether there is anything to register with a platform's runtime. */
export function hasAssets({ images, icons, fonts }: Assets): boolean {
  return images.size > 0 || icons.size > 0 || fonts.length > 0;
}
