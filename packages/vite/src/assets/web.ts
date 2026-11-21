import { createReadStream } from "node:fs";
import type { Connect } from "vite";
import { fontFaceStart } from "./font.ts";
import type { AssetFile, Assets } from "./scan.ts";

export const WEB_ASSETS_ID = "virtual:weft/assets";
export const RESOLVED_WEB_ASSETS_ID = `\0${WEB_ASSETS_ID}`;

/** Inline module whose URLs are either Vite output references or dev-server URLs. */
export function webAssets(assets: Assets, url: (file: AssetFile, output: string) => string): string {
  const code = ['import { registerAssets } from "@weft-ui/runtime";'];
  const images: string[] = [];
  const fonts: string[] = [];
  function reference(file: AssetFile, prefix = ""): string {
    const id = `u${code.length}`;
    code.push(`const ${id} = ${url(file, `weft-assets/${prefix}${file.file}`)};`);
    return id;
  }
  for (const [name, variants] of assets.images) {
    const urls = variants.map((variant) => reference(variant));
    const srcset = variants.length === 1 && variants[0]!.scale === 1 ? '""'
      : `[${variants.map(({ scale }, i) => `${urls[i]} + " ${scale}x"`).join(", ")}].join(", ")`;
    images.push(`[${JSON.stringify(name)}]: { src: ${urls[0]}, srcset: ${srcset} }`);
  }
  for (const [name, file] of assets.icons) {
    images.push(`[${JSON.stringify(name)}]: { src: ${reference(file)}, srcset: "" }`);
  }
  for (const font of assets.fonts) {
    const id = reference(font, "fonts/");
    const rule = fontFaceStart(font.family, font.weight, font.italic);
    fonts.push(`${JSON.stringify(rule)} + JSON.stringify(${id}) + "); font-display: block; }"`);
  }
  code.push(`export const manifest = { ${images.join(", ")} };`);
  code.push(`registerAssets(manifest, ${fonts.length ? `[${fonts.join(", ")}].join("\\n")` : '""'});`);
  return code.join("\n");
}

const mime: Readonly<Record<string, string>> = {
  ".png": "image/png", ".jpg": "image/jpeg", ".jpeg": "image/jpeg", ".webp": "image/webp",
  ".svg": "image/svg+xml", ".ttf": "font/ttf", ".otf": "font/otf",
};

function devFiles(assets: Assets): Map<string, AssetFile> {
  const files = new Map<string, AssetFile>();
  for (const variants of assets.images.values()) for (const file of variants) files.set(file.file, file);
  for (const file of assets.icons.values()) files.set(file.file, file);
  for (const file of assets.fonts) files.set(`fonts/${file.file}`, file);
  return files;
}

/** Serves only scanned files, including projects with a custom asset root outside Vite's root. */
export function serveWebAssets(base: string, assets: () => Assets): Connect.NextHandleFunction {
  const prefix = `${base}weft-assets/`;
  let previous: Assets | undefined;
  let files: Map<string, AssetFile>;
  return (req, res, next) => {
    const path = req.url?.split("?", 1)[0];
    if (!path?.startsWith(prefix) || (req.method !== "GET" && req.method !== "HEAD")) return next();
    let name: string;
    try {
      name = decodeURIComponent(path.slice(prefix.length));
    } catch (error) {
      return next(error);
    }
    try {
      const current = assets();
      if (previous !== current) {
        files = devFiles(current);
        previous = current;
      }
    } catch (error) {
      return next(error);
    }
    const file = files.get(name);
    if (!file) return next();
    res.setHeader("Content-Type", mime[file.ext]!);
    res.setHeader("Cache-Control", "no-store");
    if (req.method === "HEAD") res.end();
    else createReadStream(file.path).on("error", next).pipe(res);
  };
}
