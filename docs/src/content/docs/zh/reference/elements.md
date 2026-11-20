---
title: 内置元素
---
<!-- Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit. -->

除非另有说明，所有平台都支持列出的元素、属性和事件。使用标为「不支持」的功能会导致编译错误；标为「忽略」的功能可以编译，但没有效果。

所有元素还接受 `onMount` 和 `onUnmount`，这两个事件都没有载荷。

## Column

将子节点纵向排列。

**子节点:** 不限.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | 设置内容盒的宽度。 | — |
| `height` | `Int` | — | 设置内容盒的高度。 | — |
| `grow` | `Float` | `0` | 设置子元素在线性容器主轴上分得剩余空间的比例。 | — |
| `minWidth` | `Int` | — | 设置内容宽度的下限。 | — |
| `maxWidth` | `Int` | — | 设置内容宽度的上限。 | — |
| `minHeight` | `Int` | — | 设置内容高度的下限。 | — |
| `maxHeight` | `Int` | — | 设置内容高度的上限。 | — |
| `fill` | `Bool` | `false` | 在线性容器中撑满交叉轴，或在 Stack 中撑满两个轴。 | — |
| `aspectRatio` | `Float` | — | 在由一边推算另一边时指定宽高比。 | — |
| `margin` | `Int` | `0` | 设置四边的外边距。 | — |
| `marginX` | `Int` | `0` | 设置左右外边距。 | — |
| `marginY` | `Int` | `0` | 设置上下外边距。 | — |
| `marginTop` | `Int` | `0` | 设置上外边距。 | — |
| `marginBottom` | `Int` | `0` | 设置下外边距。 | — |
| `marginStart` | `Int` | `0` | 设置从左到右布局中的左外边距。 | — |
| `marginEnd` | `Int` | `0` | 设置从左到右布局中的右外边距。 | — |
| `opacity` | `Float` | `1` | 设置整个元素及其子元素的不透明度。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `hidden` | `Bool` | `false` | 隐藏元素，使其不参与布局。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `padding` | `Int` | `0` | 设置四边的内边距。 | — |
| `paddingX` | `Int` | `0` | 设置左右内边距。 | — |
| `paddingY` | `Int` | `0` | 设置上下内边距。 | — |
| `paddingTop` | `Int` | `0` | 设置上内边距。 | — |
| `paddingBottom` | `Int` | `0` | 设置下内边距。 | — |
| `paddingStart` | `Int` | `0` | 设置从左到右布局中的左内边距。 | — |
| `paddingEnd` | `Int` | `0` | 设置从左到右布局中的右内边距。 | — |
| `background` | `Color` | — | 为内边距盒填充背景色，边框下方也会着色。 | — |
| `radius` | `Int` | `0` | 为背景和边框设置圆角。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `border` | `Int` | `0` | 在内边距盒外绘制指定宽度的实线边框。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `borderColor` | `Color` | `"#0000001f"` | 设置边框颜色；深色模式默认使用 `#ffffff26`。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `shadow` | `Shadow` | `"none"` | 在边框盒外绘制预设阴影。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `clip` | `Bool` | `false` | 将溢出的内容裁剪到带圆角的内边距盒内。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `gap` | `Int` | `0` | 设置相邻子节点之间的距离。 | — |
| `align` | `CrossAlign` | `"start"` | 设置子元素在水平交叉轴上的位置。 | — |
| `justify` | `Justify` | `"start"` | 分配纵向主轴上的剩余空间。 | — |

## Row

将子节点横向排列。

**子节点:** 不限.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | 设置内容盒的宽度。 | — |
| `height` | `Int` | — | 设置内容盒的高度。 | — |
| `grow` | `Float` | `0` | 设置子元素在线性容器主轴上分得剩余空间的比例。 | — |
| `minWidth` | `Int` | — | 设置内容宽度的下限。 | — |
| `maxWidth` | `Int` | — | 设置内容宽度的上限。 | — |
| `minHeight` | `Int` | — | 设置内容高度的下限。 | — |
| `maxHeight` | `Int` | — | 设置内容高度的上限。 | — |
| `fill` | `Bool` | `false` | 在线性容器中撑满交叉轴，或在 Stack 中撑满两个轴。 | — |
| `aspectRatio` | `Float` | — | 在由一边推算另一边时指定宽高比。 | — |
| `margin` | `Int` | `0` | 设置四边的外边距。 | — |
| `marginX` | `Int` | `0` | 设置左右外边距。 | — |
| `marginY` | `Int` | `0` | 设置上下外边距。 | — |
| `marginTop` | `Int` | `0` | 设置上外边距。 | — |
| `marginBottom` | `Int` | `0` | 设置下外边距。 | — |
| `marginStart` | `Int` | `0` | 设置从左到右布局中的左外边距。 | — |
| `marginEnd` | `Int` | `0` | 设置从左到右布局中的右外边距。 | — |
| `opacity` | `Float` | `1` | 设置整个元素及其子元素的不透明度。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `hidden` | `Bool` | `false` | 隐藏元素，使其不参与布局。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `padding` | `Int` | `0` | 设置四边的内边距。 | — |
| `paddingX` | `Int` | `0` | 设置左右内边距。 | — |
| `paddingY` | `Int` | `0` | 设置上下内边距。 | — |
| `paddingTop` | `Int` | `0` | 设置上内边距。 | — |
| `paddingBottom` | `Int` | `0` | 设置下内边距。 | — |
| `paddingStart` | `Int` | `0` | 设置从左到右布局中的左内边距。 | — |
| `paddingEnd` | `Int` | `0` | 设置从左到右布局中的右内边距。 | — |
| `background` | `Color` | — | 为内边距盒填充背景色，边框下方也会着色。 | — |
| `radius` | `Int` | `0` | 为背景和边框设置圆角。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `border` | `Int` | `0` | 在内边距盒外绘制指定宽度的实线边框。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `borderColor` | `Color` | `"#0000001f"` | 设置边框颜色；深色模式默认使用 `#ffffff26`。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `shadow` | `Shadow` | `"none"` | 在边框盒外绘制预设阴影。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `clip` | `Bool` | `false` | 将溢出的内容裁剪到带圆角的内边距盒内。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `gap` | `Int` | `0` | 设置相邻子节点之间的距离。 | — |
| `align` | `CrossAlign` | `"start"` | 设置子元素在纵向交叉轴上的位置。 | — |
| `justify` | `Justify` | `"start"` | 分配横向主轴上的剩余空间。 | — |

## Text

显示文本。

**子节点:** 无.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | 设置内容盒的宽度。 | — |
| `height` | `Int` | — | 设置内容盒的高度。 | — |
| `grow` | `Float` | `0` | 设置子元素在线性容器主轴上分得剩余空间的比例。 | — |
| `minWidth` | `Int` | — | 设置内容宽度的下限。 | — |
| `maxWidth` | `Int` | — | 设置内容宽度的上限。 | — |
| `minHeight` | `Int` | — | 设置内容高度的下限。 | — |
| `maxHeight` | `Int` | — | 设置内容高度的上限。 | — |
| `fill` | `Bool` | `false` | 在线性容器中撑满交叉轴，或在 Stack 中撑满两个轴。 | — |
| `aspectRatio` | `Float` | — | 在由一边推算另一边时指定宽高比。 | — |
| `margin` | `Int` | `0` | 设置四边的外边距。 | — |
| `marginX` | `Int` | `0` | 设置左右外边距。 | — |
| `marginY` | `Int` | `0` | 设置上下外边距。 | — |
| `marginTop` | `Int` | `0` | 设置上外边距。 | — |
| `marginBottom` | `Int` | `0` | 设置下外边距。 | — |
| `marginStart` | `Int` | `0` | 设置从左到右布局中的左外边距。 | — |
| `marginEnd` | `Int` | `0` | 设置从左到右布局中的右外边距。 | — |
| `opacity` | `Float` | `1` | 设置整个元素及其子元素的不透明度。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `hidden` | `Bool` | `false` | 隐藏元素，使其不参与布局。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `padding` | `Int` | `0` | 设置四边的内边距。 | — |
| `paddingX` | `Int` | `0` | 设置左右内边距。 | — |
| `paddingY` | `Int` | `0` | 设置上下内边距。 | — |
| `paddingTop` | `Int` | `0` | 设置上内边距。 | — |
| `paddingBottom` | `Int` | `0` | 设置下内边距。 | — |
| `paddingStart` | `Int` | `0` | 设置从左到右布局中的左内边距。 | — |
| `paddingEnd` | `Int` | `0` | 设置从左到右布局中的右内边距。 | — |
| `background` | `Color` | — | 为内边距盒填充背景色，边框下方也会着色。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `radius` | `Int` | `0` | 为背景和边框设置圆角。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `border` | `Int` | `0` | 在内边距盒外绘制指定宽度的实线边框。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `borderColor` | `Color` | `"#0000001f"` | 设置边框颜色；深色模式默认使用 `#ffffff26`。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `shadow` | `Shadow` | `"none"` | 在边框盒外绘制预设阴影。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `clip` | `Bool` | `false` | 将溢出的内容裁剪到带圆角的内边距盒内。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `value` (位置参数) | `String` | 必填 | 要显示的文本。 | — |
| `color` | `Color` | — | 设置文字颜色。 | — |
| `size` | `Int` | — | 用逻辑像素设置字号。 | — |
| `bold` | `Bool` | — | 以粗体显示文本。 | — |

## Button

显示可点击的控件，用子节点提供按钮内容。

**子节点:** 不限.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | 设置内容盒的宽度。 | — |
| `height` | `Int` | — | 设置内容盒的高度。 | — |
| `grow` | `Float` | `0` | 设置子元素在线性容器主轴上分得剩余空间的比例。 | — |
| `minWidth` | `Int` | — | 设置内容宽度的下限。 | — |
| `maxWidth` | `Int` | — | 设置内容宽度的上限。 | — |
| `minHeight` | `Int` | — | 设置内容高度的下限。 | — |
| `maxHeight` | `Int` | — | 设置内容高度的上限。 | — |
| `fill` | `Bool` | `false` | 在线性容器中撑满交叉轴，或在 Stack 中撑满两个轴。 | — |
| `aspectRatio` | `Float` | — | 在由一边推算另一边时指定宽高比。 | — |
| `margin` | `Int` | `0` | 设置四边的外边距。 | — |
| `marginX` | `Int` | `0` | 设置左右外边距。 | — |
| `marginY` | `Int` | `0` | 设置上下外边距。 | — |
| `marginTop` | `Int` | `0` | 设置上外边距。 | — |
| `marginBottom` | `Int` | `0` | 设置下外边距。 | — |
| `marginStart` | `Int` | `0` | 设置从左到右布局中的左外边距。 | — |
| `marginEnd` | `Int` | `0` | 设置从左到右布局中的右外边距。 | — |
| `opacity` | `Float` | `1` | 设置整个元素及其子元素的不透明度。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `hidden` | `Bool` | `false` | 隐藏元素，使其不参与布局。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `disabled` | `Bool` | — | 禁用按钮。 | — |

| 事件 | 载荷 | 说明 | 平台例外 |
| --- | --- | --- | --- |
| `onClick` | — | 按钮被点击时触发。 | — |

## Input

显示可编辑的文本框。

**子节点:** 无.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | 设置内容盒的宽度。 | — |
| `height` | `Int` | — | 设置内容盒的高度。 | — |
| `grow` | `Float` | `0` | 设置子元素在线性容器主轴上分得剩余空间的比例。 | — |
| `minWidth` | `Int` | — | 设置内容宽度的下限。 | — |
| `maxWidth` | `Int` | — | 设置内容宽度的上限。 | — |
| `minHeight` | `Int` | — | 设置内容高度的下限。 | — |
| `maxHeight` | `Int` | — | 设置内容高度的上限。 | — |
| `fill` | `Bool` | `false` | 在线性容器中撑满交叉轴，或在 Stack 中撑满两个轴。 | — |
| `aspectRatio` | `Float` | — | 在由一边推算另一边时指定宽高比。 | — |
| `margin` | `Int` | `0` | 设置四边的外边距。 | — |
| `marginX` | `Int` | `0` | 设置左右外边距。 | — |
| `marginY` | `Int` | `0` | 设置上下外边距。 | — |
| `marginTop` | `Int` | `0` | 设置上外边距。 | — |
| `marginBottom` | `Int` | `0` | 设置下外边距。 | — |
| `marginStart` | `Int` | `0` | 设置从左到右布局中的左外边距。 | — |
| `marginEnd` | `Int` | `0` | 设置从左到右布局中的右外边距。 | — |
| `opacity` | `Float` | `1` | 设置整个元素及其子元素的不透明度。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `hidden` | `Bool` | `false` | 隐藏元素，使其不参与布局。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `value` (位置参数) | `String` | 必填 | 当前输入的文本。 | — |
| `placeholder` | `String` | — | 输入框为空时显示提示文字。 | — |
| `disabled` | `Bool` | — | 禁用输入框。 | — |

| 事件 | 载荷 | 说明 | 平台例外 |
| --- | --- | --- | --- |
| `onChange` | `String` | 编辑后提供修改后的文本。 | — |

## Checkbox

显示可勾选的控件。

**子节点:** 无.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | 设置内容盒的宽度。 | — |
| `height` | `Int` | — | 设置内容盒的高度。 | — |
| `grow` | `Float` | `0` | 设置子元素在线性容器主轴上分得剩余空间的比例。 | — |
| `minWidth` | `Int` | — | 设置内容宽度的下限。 | — |
| `maxWidth` | `Int` | — | 设置内容宽度的上限。 | — |
| `minHeight` | `Int` | — | 设置内容高度的下限。 | — |
| `maxHeight` | `Int` | — | 设置内容高度的上限。 | — |
| `fill` | `Bool` | `false` | 在线性容器中撑满交叉轴，或在 Stack 中撑满两个轴。 | — |
| `aspectRatio` | `Float` | — | 在由一边推算另一边时指定宽高比。 | — |
| `margin` | `Int` | `0` | 设置四边的外边距。 | — |
| `marginX` | `Int` | `0` | 设置左右外边距。 | — |
| `marginY` | `Int` | `0` | 设置上下外边距。 | — |
| `marginTop` | `Int` | `0` | 设置上外边距。 | — |
| `marginBottom` | `Int` | `0` | 设置下外边距。 | — |
| `marginStart` | `Int` | `0` | 设置从左到右布局中的左外边距。 | — |
| `marginEnd` | `Int` | `0` | 设置从左到右布局中的右外边距。 | — |
| `opacity` | `Float` | `1` | 设置整个元素及其子元素的不透明度。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `hidden` | `Bool` | `false` | 隐藏元素，使其不参与布局。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `checked` (位置参数) | `Bool` | 必填 | 当前的选中状态。 | — |
| `disabled` | `Bool` | — | 禁用复选框。 | — |

| 事件 | 载荷 | 说明 | 平台例外 |
| --- | --- | --- | --- |
| `onChange` | `Bool` | 提供修改后的选中状态。 | — |

## Image

显示指定 URL 的图片。

**子节点:** 无.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | 设置内容盒的宽度。 | — |
| `height` | `Int` | — | 设置内容盒的高度。 | — |
| `grow` | `Float` | `0` | 设置子元素在线性容器主轴上分得剩余空间的比例。 | — |
| `minWidth` | `Int` | — | 设置内容宽度的下限。 | — |
| `maxWidth` | `Int` | — | 设置内容宽度的上限。 | — |
| `minHeight` | `Int` | — | 设置内容高度的下限。 | — |
| `maxHeight` | `Int` | — | 设置内容高度的上限。 | — |
| `fill` | `Bool` | `false` | 在线性容器中撑满交叉轴，或在 Stack 中撑满两个轴。 | — |
| `aspectRatio` | `Float` | — | 在由一边推算另一边时指定宽高比。 | — |
| `margin` | `Int` | `0` | 设置四边的外边距。 | — |
| `marginX` | `Int` | `0` | 设置左右外边距。 | — |
| `marginY` | `Int` | `0` | 设置上下外边距。 | — |
| `marginTop` | `Int` | `0` | 设置上外边距。 | — |
| `marginBottom` | `Int` | `0` | 设置下外边距。 | — |
| `marginStart` | `Int` | `0` | 设置从左到右布局中的左外边距。 | — |
| `marginEnd` | `Int` | `0` | 设置从左到右布局中的右外边距。 | — |
| `opacity` | `Float` | `1` | 设置整个元素及其子元素的不透明度。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `hidden` | `Bool` | `false` | 隐藏元素，使其不参与布局。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `background` | `Color` | — | 为内边距盒填充背景色，边框下方也会着色。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `radius` | `Int` | `0` | 为背景和边框设置圆角。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `border` | `Int` | `0` | 在内边距盒外绘制指定宽度的实线边框。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `borderColor` | `Color` | `"#0000001f"` | 设置边框颜色；深色模式默认使用 `#ffffff26`。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `shadow` | `Shadow` | `"none"` | 在边框盒外绘制预设阴影。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `clip` | `Bool` | `false` | 将溢出的内容裁剪到带圆角的内边距盒内。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `src` (位置参数) | `String` | 必填 | 图片的 URL。 | — |

## Stack

按声明顺序叠放子元素。

**子节点:** 不限.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | 设置内容盒的宽度。 | — |
| `height` | `Int` | — | 设置内容盒的高度。 | — |
| `grow` | `Float` | `0` | 设置子元素在线性容器主轴上分得剩余空间的比例。 | — |
| `minWidth` | `Int` | — | 设置内容宽度的下限。 | — |
| `maxWidth` | `Int` | — | 设置内容宽度的上限。 | — |
| `minHeight` | `Int` | — | 设置内容高度的下限。 | — |
| `maxHeight` | `Int` | — | 设置内容高度的上限。 | — |
| `fill` | `Bool` | `false` | 在线性容器中撑满交叉轴，或在 Stack 中撑满两个轴。 | — |
| `aspectRatio` | `Float` | — | 在由一边推算另一边时指定宽高比。 | — |
| `margin` | `Int` | `0` | 设置四边的外边距。 | — |
| `marginX` | `Int` | `0` | 设置左右外边距。 | — |
| `marginY` | `Int` | `0` | 设置上下外边距。 | — |
| `marginTop` | `Int` | `0` | 设置上外边距。 | — |
| `marginBottom` | `Int` | `0` | 设置下外边距。 | — |
| `marginStart` | `Int` | `0` | 设置从左到右布局中的左外边距。 | — |
| `marginEnd` | `Int` | `0` | 设置从左到右布局中的右外边距。 | — |
| `opacity` | `Float` | `1` | 设置整个元素及其子元素的不透明度。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `hidden` | `Bool` | `false` | 隐藏元素，使其不参与布局。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `padding` | `Int` | `0` | 设置四边的内边距。 | — |
| `paddingX` | `Int` | `0` | 设置左右内边距。 | — |
| `paddingY` | `Int` | `0` | 设置上下内边距。 | — |
| `paddingTop` | `Int` | `0` | 设置上内边距。 | — |
| `paddingBottom` | `Int` | `0` | 设置下内边距。 | — |
| `paddingStart` | `Int` | `0` | 设置从左到右布局中的左内边距。 | — |
| `paddingEnd` | `Int` | `0` | 设置从左到右布局中的右内边距。 | — |
| `background` | `Color` | — | 为内边距盒填充背景色，边框下方也会着色。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `radius` | `Int` | `0` | 为背景和边框设置圆角。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `border` | `Int` | `0` | 在内边距盒外绘制指定宽度的实线边框。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `borderColor` | `Color` | `"#0000001f"` | 设置边框颜色；深色模式默认使用 `#ffffff26`。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `shadow` | `Shadow` | `"none"` | 在边框盒外绘制预设阴影。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `clip` | `Bool` | `false` | 将溢出的内容裁剪到带圆角的内边距盒内。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `alignment` | `Alignment` | `"topStart"` | 设置子元素在堆叠容器两个轴上的位置。 | — |

## Scroll

线性排列子元素，并可沿主轴滚动。

**子节点:** 不限.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | 设置内容盒的宽度。 | — |
| `height` | `Int` | — | 设置内容盒的高度。 | — |
| `grow` | `Float` | `0` | 设置子元素在线性容器主轴上分得剩余空间的比例。 | — |
| `minWidth` | `Int` | — | 设置内容宽度的下限。 | — |
| `maxWidth` | `Int` | — | 设置内容宽度的上限。 | — |
| `minHeight` | `Int` | — | 设置内容高度的下限。 | — |
| `maxHeight` | `Int` | — | 设置内容高度的上限。 | — |
| `fill` | `Bool` | `false` | 在线性容器中撑满交叉轴，或在 Stack 中撑满两个轴。 | — |
| `aspectRatio` | `Float` | — | 在由一边推算另一边时指定宽高比。 | — |
| `margin` | `Int` | `0` | 设置四边的外边距。 | — |
| `marginX` | `Int` | `0` | 设置左右外边距。 | — |
| `marginY` | `Int` | `0` | 设置上下外边距。 | — |
| `marginTop` | `Int` | `0` | 设置上外边距。 | — |
| `marginBottom` | `Int` | `0` | 设置下外边距。 | — |
| `marginStart` | `Int` | `0` | 设置从左到右布局中的左外边距。 | — |
| `marginEnd` | `Int` | `0` | 设置从左到右布局中的右外边距。 | — |
| `opacity` | `Float` | `1` | 设置整个元素及其子元素的不透明度。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `hidden` | `Bool` | `false` | 隐藏元素，使其不参与布局。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `padding` | `Int` | `0` | 设置四边的内边距。 | — |
| `paddingX` | `Int` | `0` | 设置左右内边距。 | — |
| `paddingY` | `Int` | `0` | 设置上下内边距。 | — |
| `paddingTop` | `Int` | `0` | 设置上内边距。 | — |
| `paddingBottom` | `Int` | `0` | 设置下内边距。 | — |
| `paddingStart` | `Int` | `0` | 设置从左到右布局中的左内边距。 | — |
| `paddingEnd` | `Int` | `0` | 设置从左到右布局中的右内边距。 | — |
| `background` | `Color` | — | 为内边距盒填充背景色，边框下方也会着色。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `radius` | `Int` | `0` | 为背景和边框设置圆角。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `border` | `Int` | `0` | 在内边距盒外绘制指定宽度的实线边框。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `borderColor` | `Color` | `"#0000001f"` | 设置边框颜色；深色模式默认使用 `#ffffff26`。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `shadow` | `Shadow` | `"none"` | 在边框盒外绘制预设阴影。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `clip` | `Bool` | `false` | 将溢出的内容裁剪到带圆角的内边距盒内。 | 不支持: `ios`; 不支持: `macos`; 不支持: `android`; 不支持: `windows`; 不支持: `linux` |
| `direction` | `Axis` | `"vertical"` | 在编译时选择滚动轴。 | — |
| `gap` | `Int` | `0` | 设置相邻子节点之间的距离。 | — |
| `align` | `CrossAlign` | `"start"` | 设置子元素在非滚动交叉轴上的位置。 | — |
| `indicators` | `Bool` | `true` | 控制是否显示滚动条。 | — |

## Spacer

占用父容器主轴空间，不绘制内容。

**子节点:** 无.

必须是以下元素之一的直接子元素： `Column`, `Row`, `Scroll`.

| 属性 | 类型 | 默认值 | 说明 | 平台例外 |
| --- | --- | --- | --- | --- |
| `size` | `Int` | — | 占用固定长度；未设置时，参与分配剩余空间。 | — |
