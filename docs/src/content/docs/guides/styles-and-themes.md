---
title: Styles and themes
description: Reuse element styles and define colors that follow system appearance.
---

A `style` gives named properties on built-in elements reusable defaults:

```weft
style Muted { color = "#6b7280", size = 12 }

component Note(message: String) {
  Text(message, style = Muted, size = 14)
}
```

`Note` uses the color from `Muted` and the explicit size `14`. An element can apply one style, and it must accept every property in that style. A style cannot set positional properties or events, include other styles, or compute values.

## Define the theme

Put shared colors and dimensions in a top-level `theme` block. Only one theme can be declared across an app's imports.

```weft
// Palette.weft
theme {
  accent = "#0a84ff" dark "#409cff",
  surface = "#ffffff" dark "#1c1c1e",
  labelSize = 16,
  gap = 8,
}
```

Reference a token as `theme.name` in the declaring file or in a file that directly imports it. Styles resolve their token references in the file where the style is declared.

```weft
import "./Palette.weft"

style AccentLabel { color = theme.accent, size = theme.labelSize }

component Panel() {
  Column(background = theme.surface, gap = theme.gap) {
    Text("Settings", style = AccentLabel)
  }
}
```

The color before `dark` is the light appearance; omit `dark` to use the same color in both modes. `Int` and `Float` tokens always have one value, substituted at compile time. Colors follow the system appearance automatically; there is no Weft setting to force a mode.

Theme tokens work on every platform and follow the system appearance while the app runs:

- **Web**: CSS variables and the `prefers-color-scheme` media query.
- **WeChat mini programs**: the same CSS variables. New mini program projects opt in to WeChat dark mode automatically. If your project already has an `app.json`, add `"darkmode": true` to it.
- **iOS and macOS**: dynamic `UIColor` / `NSColor` values.
- **Android**: colors are resolved during composition from `isSystemInDarkTheme()`.
- **Windows**: colors are reapplied when the window's `ActualTheme` changes.
- **Linux**: the color scheme comes from the XDG desktop portal (`org.freedesktop.appearance` `color-scheme`). Without a portal, apps use light colors.

See the [element and feature support matrix](/reference/elements/).

## Pass a `Color` through a component

`Color` is a Weft type for color tokens or hexadecimal colors. Use it in component parameters, state, model fields, or callbacks:

```weft
component Badge(label: String, tint: Color) {
  Text(label, color = tint)
}
```

Pass `theme.accent` or a hexadecimal string (`#RGB`, `#RRGGBB`, or `#RRGGBBAA`). Invalid literals are compile errors. Dynamic invalid colors render transparent and warn in development builds. `Color` cannot be assigned back to `String`, compared, or interpolated. In TypeScript declarations it appears as `string`.
