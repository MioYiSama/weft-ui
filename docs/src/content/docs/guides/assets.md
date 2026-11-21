---
title: Bundle images, icons and fonts
description: Package local assets and use them in Weft UI views.
---

Bundled images, SVG icons, and fonts work on every target platform. URL images do not need the asset pipeline.

## Images and icons

Create an `assets/` directory next to your Vite config (relative to Vite's `root` if you set one):

```text
assets/
  images/
    hero.jpg
    hero@2x.jpg
    hero@3x.jpg
  icons/
    compass.svg
  fonts/
    Inter-Regular.ttf
    Inter-Bold.ttf
```

Use `.png`, `.jpg`, `.jpeg`, or `.webp` for raster images and `.svg` for icons. The image or icon name may contain only lowercase `a`–`z`, `0`–`9`, and `_`. An image and an icon may not share a name. Image density variants use `@2x` and `@3x`; the regular 1× file is optional. Reference a bundled asset without its extension or density suffix:

```weft
Image("asset:hero", width = 180, height = 90, fit = "contain",
      placeholder = "#e5efee")
Image("asset:compass", width = 24, height = 24)
```

`cover` (the default) preserves the image's proportions and crops it to fill the box. `contain` shows the entire image, leaving space around it when the proportions differ. `fill` stretches to fit. Without a width or height, the image uses its intrinsic size in logical pixels; specifying only one dimension preserves the aspect ratio. `placeholder` is the background color while an image loads or if it fails. A missing asset is not a build error; debug builds warn at runtime.

Web builds emit files under `weft-assets/`, respecting Vite's `base`; the browser chooses a density via `srcset`. WeChat packages the files under `weft/assets/` and uses the smallest density at least as large as the window's pixel ratio, or the largest if none is high enough.

## Fonts

Put `.ttf` or `.otf` files in `assets/fonts/`. The filename must end in `-Light`, `-Regular`, `-Medium`, `-SemiBold`, or `-Bold`, followed by `Italic` for an italic face (for example, `Inter-RegularItalic.ttf`). Weft reads the family name from the font itself, not its filename. Set `font` on `Text` or `Span`, and use `weight` and `italic` to pick a face:

```weft
Text("North ridge", font = "Inter", weight = "bold")
```

WeChat embeds fonts as base64, which increases the main package size. The plugin warns when the encoded font data exceeds 512 KiB. Include only the faces you use and make sure their licenses allow bundling.

## Use another asset directory

Set `assets` in `weft()` to override the default. Changes in the directory reload the Web dev page automatically.

```ts
weft({ assets: "src/media" })
```
