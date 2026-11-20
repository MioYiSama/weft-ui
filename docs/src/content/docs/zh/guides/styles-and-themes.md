---
title: 样式与主题
description: 复用元素样式，定义随系统深浅色外观切换的颜色。
---

用 `style` 为内置元素的具名属性设置一组可复用的默认值：

```weft
style Muted { color = "#6b7280", size = 12 }

component Note(message: String) {
  Text(message, style = Muted, size = 14)
}
```

`Note` 的颜色来自 `Muted`，字号使用显式传入的 `14`。一个元素只能应用一个样式，样式里的每个属性都必须是该元素接受的属性。样式不能设置位置属性或事件，也不能引用其他样式或使用计算表达式。

## 定义主题

把共用的颜色、尺寸放到顶层 `theme` 中。整个应用的导入闭包只能声明一个主题。

```weft
// Palette.weft
theme {
  accent = "#0a84ff" dark "#409cff",
  surface = "#ffffff" dark "#1c1c1e",
  labelSize = 16,
  gap = 8,
}
```

使用 `theme.name` 引用令牌。引用只能写在声明主题的文件中，或写在直接导入它的文件中。样式中的令牌在声明样式的文件里解析。

```weft
import "./Palette.weft"

style AccentLabel { color = theme.accent, size = theme.labelSize }

component Panel() {
  Column(background = theme.surface, gap = theme.gap) {
    Text("设置", style = AccentLabel)
  }
}
```

`dark` 前的颜色用于浅色外观；不写 `dark` 时，深浅色共用同一个值。`Int` 和 `Float` 令牌只有一个值，在编译期替换为字面量。颜色自动跟随系统外观，Weft 不提供应用内切换开关。

目前 Web 和微信小程序支持主题令牌。新建的小程序项目已默认开启微信深色模式；已有项目可在 `app.json` 中加上 `"darkmode": true`。各平台的支持情况见[元素与功能参考](/zh/reference/elements/)。

## 在组件间传递 `Color`

`Color` 是 Weft 的颜色类型，可以表示主题颜色或十六进制颜色。组件参数、状态、模型字段和回调都可以使用它：

```weft
component Badge(label: String, tint: Color) {
  Text(label, color = tint)
}
```

可以传入 `theme.accent`，也可以传入 `#RGB`、`#RRGGBB` 或 `#RRGGBBAA` 格式的字符串。字面量不合法时，编译器会报错；无效的动态颜色显示为透明，开发构建还会发出警告。`Color` 不能赋回 `String`，也不能比较或插值。在 TypeScript 声明中它表示为 `string`。
