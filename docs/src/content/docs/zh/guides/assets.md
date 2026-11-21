---
title: 打包图片、图标和字体
description: 打包本地资源并在 Weft UI 视图中使用。
---

本地图片、SVG 图标和字体目前支持 Web 和微信小程序。直接使用 URL 的图片不需要打包。

## 图片与图标

在 Vite 项目根目录下创建 `assets/`。如果设置了 Vite 的 `root`，则以它为准：

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

位图支持 `.png`、`.jpg`、`.jpeg` 和 `.webp`，图标使用 `.svg`。图片名和图标名只能包含小写字母 `a`–`z`、数字 `0`–`9` 和下划线 `_`，两者不能重名。位图可用 `@2x` 和 `@3x` 提供高密度版本，不带后缀的 1× 文件可以省略。引用时不写扩展名和密度后缀：

```weft
Image("asset:hero", width = 180, height = 90, fit = "contain",
      placeholder = "#e5efee")
Image("asset:compass", width = 24, height = 24)
```

`cover`（默认）等比缩放，填满盒子并裁掉超出部分；`contain` 完整显示图片，宽高比不同时留白；`fill` 拉伸到盒子大小。不指定宽高时，使用以逻辑像素计量的固有尺寸；只指定一边时，另一边按图片比例计算。`placeholder` 是加载中或加载失败时的背景色。找不到资源不会导致构建失败，调试构建会在运行时发出警告。

Web 将资源写到 `weft-assets/`，URL 遵循 Vite 的 `base`，浏览器通过 `srcset` 选择密度。小程序将图片打包到 `weft/assets/`，选择不低于窗口像素比的最小密度；如果都低于窗口像素比，则选择最高密度。

## 字体

把 `.ttf` 或 `.otf` 文件放在 `assets/fonts/`。扩展名前必须以 `-Light`、`-Regular`、`-Medium`、`-SemiBold` 或 `-Bold` 结尾，斜体在字重后加 `Italic`（例如 `Inter-RegularItalic.ttf`）。Weft 从字体文件中读取族名，而不是从文件名推断。在 `Text` 或 `Span` 上指定 `font`，通过 `weight` 和 `italic` 选择字形：

```weft
Text("北山脊", font = "Inter", weight = "bold")
```

小程序用 base64 嵌入字体，会增加主包体积。编码后的字体数据超过 512 KiB 时，插件会发出警告。只打包用到的字形，并确认字体许可证允许打包分发。

## 自定义资源目录

通过 `weft()` 的 `assets` 选项修改默认目录。开发时，资源变化会自动刷新 Web 页面。

```ts
weft({ assets: "src/media" })
```
