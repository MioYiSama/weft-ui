---
title: Built-in elements
---
<!-- Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit. -->

Elements, properties, and events are supported on all target platforms unless noted. Unsupported uses cause a compilation error; ignored uses are accepted but have no effect.

All elements also accept `onMount` and `onUnmount`, which have no payload.

## Column

Stacks children vertically.

**Children:** Any.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | Sets the width of the content box. | — |
| `height` | `Int` | — | Sets the height of the content box. | — |
| `grow` | `Float` | `0` | Sets the share of remaining main-axis space for a child in a linear container. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minWidth` | `Int` | — | Sets the minimum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxWidth` | `Int` | — | Sets the maximum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minHeight` | `Int` | — | Sets the minimum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxHeight` | `Int` | — | Sets the maximum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `fill` | `Bool` | `false` | Fills the cross axis in a linear container or both axes in a Stack. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `aspectRatio` | `Float` | — | Sets width divided by height when one dimension determines the other. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `margin` | `Int` | `0` | Sets the outer spacing on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginX` | `Int` | `0` | Sets the outer spacing on the left and right. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginY` | `Int` | `0` | Sets the outer spacing on the top and bottom. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginTop` | `Int` | `0` | Sets the top margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginBottom` | `Int` | `0` | Sets the bottom margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginStart` | `Int` | `0` | Sets the left margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginEnd` | `Int` | `0` | Sets the right margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `padding` | `Int` | `0` | Sets the inset on all four sides. | — |
| `paddingX` | `Int` | `0` | Sets the left and right insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingY` | `Int` | `0` | Sets the top and bottom insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingTop` | `Int` | `0` | Sets the top inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingBottom` | `Int` | `0` | Sets the bottom inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingStart` | `Int` | `0` | Sets the left inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingEnd` | `Int` | `0` | Sets the right inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `background` | `Color` | — | Sets the background color. | — |
| `gap` | `Int` | `0` | Adds space between neighboring children. | — |
| `align` | `CrossAlign` | `"start"` | Positions children on the horizontal cross axis. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `justify` | `Justify` | `"start"` | Distributes remaining space on the vertical main axis. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |

## Row

Places children side by side.

**Children:** Any.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | Sets the width of the content box. | — |
| `height` | `Int` | — | Sets the height of the content box. | — |
| `grow` | `Float` | `0` | Sets the share of remaining main-axis space for a child in a linear container. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minWidth` | `Int` | — | Sets the minimum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxWidth` | `Int` | — | Sets the maximum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minHeight` | `Int` | — | Sets the minimum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxHeight` | `Int` | — | Sets the maximum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `fill` | `Bool` | `false` | Fills the cross axis in a linear container or both axes in a Stack. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `aspectRatio` | `Float` | — | Sets width divided by height when one dimension determines the other. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `margin` | `Int` | `0` | Sets the outer spacing on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginX` | `Int` | `0` | Sets the outer spacing on the left and right. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginY` | `Int` | `0` | Sets the outer spacing on the top and bottom. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginTop` | `Int` | `0` | Sets the top margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginBottom` | `Int` | `0` | Sets the bottom margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginStart` | `Int` | `0` | Sets the left margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginEnd` | `Int` | `0` | Sets the right margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `padding` | `Int` | `0` | Sets the inset on all four sides. | — |
| `paddingX` | `Int` | `0` | Sets the left and right insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingY` | `Int` | `0` | Sets the top and bottom insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingTop` | `Int` | `0` | Sets the top inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingBottom` | `Int` | `0` | Sets the bottom inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingStart` | `Int` | `0` | Sets the left inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingEnd` | `Int` | `0` | Sets the right inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `background` | `Color` | — | Sets the background color. | — |
| `gap` | `Int` | `0` | Adds space between neighboring children. | — |
| `align` | `CrossAlign` | `"start"` | Positions children on the vertical cross axis. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `justify` | `Justify` | `"start"` | Distributes remaining space on the horizontal main axis. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |

## Text

Displays text.

**Children:** None.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | Sets the width of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `height` | `Int` | — | Sets the height of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `grow` | `Float` | `0` | Sets the share of remaining main-axis space for a child in a linear container. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minWidth` | `Int` | — | Sets the minimum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxWidth` | `Int` | — | Sets the maximum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minHeight` | `Int` | — | Sets the minimum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxHeight` | `Int` | — | Sets the maximum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `fill` | `Bool` | `false` | Fills the cross axis in a linear container or both axes in a Stack. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `aspectRatio` | `Float` | — | Sets width divided by height when one dimension determines the other. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `margin` | `Int` | `0` | Sets the outer spacing on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginX` | `Int` | `0` | Sets the outer spacing on the left and right. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginY` | `Int` | `0` | Sets the outer spacing on the top and bottom. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginTop` | `Int` | `0` | Sets the top margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginBottom` | `Int` | `0` | Sets the bottom margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginStart` | `Int` | `0` | Sets the left margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginEnd` | `Int` | `0` | Sets the right margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `padding` | `Int` | `0` | Sets the inset on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingX` | `Int` | `0` | Sets the left and right insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingY` | `Int` | `0` | Sets the top and bottom insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingTop` | `Int` | `0` | Sets the top inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingBottom` | `Int` | `0` | Sets the bottom inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingStart` | `Int` | `0` | Sets the left inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingEnd` | `Int` | `0` | Sets the right inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `value` (positional) | `String` | Required | The text to display. | — |
| `color` | `Color` | — | Sets the text color. | — |
| `size` | `Int` | — | Sets the font size in logical pixels. | — |
| `bold` | `Bool` | — | Displays the text in bold. | — |

## Button

Renders a clickable control with child nodes for its content.

**Children:** Any.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | Sets the width of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `height` | `Int` | — | Sets the height of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `grow` | `Float` | `0` | Sets the share of remaining main-axis space for a child in a linear container. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minWidth` | `Int` | — | Sets the minimum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxWidth` | `Int` | — | Sets the maximum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minHeight` | `Int` | — | Sets the minimum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxHeight` | `Int` | — | Sets the maximum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `fill` | `Bool` | `false` | Fills the cross axis in a linear container or both axes in a Stack. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `aspectRatio` | `Float` | — | Sets width divided by height when one dimension determines the other. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `margin` | `Int` | `0` | Sets the outer spacing on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginX` | `Int` | `0` | Sets the outer spacing on the left and right. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginY` | `Int` | `0` | Sets the outer spacing on the top and bottom. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginTop` | `Int` | `0` | Sets the top margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginBottom` | `Int` | `0` | Sets the bottom margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginStart` | `Int` | `0` | Sets the left margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginEnd` | `Int` | `0` | Sets the right margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `disabled` | `Bool` | — | Disables the button. | — |

| Event | Payload | Description | Platform exceptions |
| --- | --- | --- | --- |
| `onClick` | — | Fires when the button is clicked. | — |

## Input

Displays an editable text field.

**Children:** None.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | Sets the width of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `height` | `Int` | — | Sets the height of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `grow` | `Float` | `0` | Sets the share of remaining main-axis space for a child in a linear container. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minWidth` | `Int` | — | Sets the minimum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxWidth` | `Int` | — | Sets the maximum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minHeight` | `Int` | — | Sets the minimum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxHeight` | `Int` | — | Sets the maximum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `fill` | `Bool` | `false` | Fills the cross axis in a linear container or both axes in a Stack. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `aspectRatio` | `Float` | — | Sets width divided by height when one dimension determines the other. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `margin` | `Int` | `0` | Sets the outer spacing on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginX` | `Int` | `0` | Sets the outer spacing on the left and right. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginY` | `Int` | `0` | Sets the outer spacing on the top and bottom. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginTop` | `Int` | `0` | Sets the top margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginBottom` | `Int` | `0` | Sets the bottom margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginStart` | `Int` | `0` | Sets the left margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginEnd` | `Int` | `0` | Sets the right margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `value` (positional) | `String` | Required | The current text. | — |
| `placeholder` | `String` | — | Shows a hint when the field is empty. | — |
| `disabled` | `Bool` | — | Disables the input field. | — |

| Event | Payload | Description | Platform exceptions |
| --- | --- | --- | --- |
| `onChange` | `String` | Reports the new text after an edit. | — |

## Checkbox

Displays a checkable control.

**Children:** None.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | Sets the width of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `height` | `Int` | — | Sets the height of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `grow` | `Float` | `0` | Sets the share of remaining main-axis space for a child in a linear container. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minWidth` | `Int` | — | Sets the minimum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxWidth` | `Int` | — | Sets the maximum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minHeight` | `Int` | — | Sets the minimum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxHeight` | `Int` | — | Sets the maximum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `fill` | `Bool` | `false` | Fills the cross axis in a linear container or both axes in a Stack. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `aspectRatio` | `Float` | — | Sets width divided by height when one dimension determines the other. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `margin` | `Int` | `0` | Sets the outer spacing on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginX` | `Int` | `0` | Sets the outer spacing on the left and right. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginY` | `Int` | `0` | Sets the outer spacing on the top and bottom. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginTop` | `Int` | `0` | Sets the top margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginBottom` | `Int` | `0` | Sets the bottom margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginStart` | `Int` | `0` | Sets the left margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginEnd` | `Int` | `0` | Sets the right margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `checked` (positional) | `Bool` | Required | The current checked state. | — |
| `disabled` | `Bool` | — | Disables the checkbox. | — |

| Event | Payload | Description | Platform exceptions |
| --- | --- | --- | --- |
| `onChange` | `Bool` | Reports the new checked state. | — |

## Image

Displays an image from a URL.

**Children:** None.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | Sets the width of the content box. | — |
| `height` | `Int` | — | Sets the height of the content box. | — |
| `grow` | `Float` | `0` | Sets the share of remaining main-axis space for a child in a linear container. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minWidth` | `Int` | — | Sets the minimum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxWidth` | `Int` | — | Sets the maximum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minHeight` | `Int` | — | Sets the minimum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxHeight` | `Int` | — | Sets the maximum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `fill` | `Bool` | `false` | Fills the cross axis in a linear container or both axes in a Stack. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `aspectRatio` | `Float` | — | Sets width divided by height when one dimension determines the other. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `margin` | `Int` | `0` | Sets the outer spacing on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginX` | `Int` | `0` | Sets the outer spacing on the left and right. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginY` | `Int` | `0` | Sets the outer spacing on the top and bottom. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginTop` | `Int` | `0` | Sets the top margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginBottom` | `Int` | `0` | Sets the bottom margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginStart` | `Int` | `0` | Sets the left margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginEnd` | `Int` | `0` | Sets the right margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `src` (positional) | `String` | Required | The image URL. | — |

## Stack

Overlays children in declaration order.

**Platform exceptions:** Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux`.

**Children:** Any.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | Sets the width of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `height` | `Int` | — | Sets the height of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `grow` | `Float` | `0` | Sets the share of remaining main-axis space for a child in a linear container. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minWidth` | `Int` | — | Sets the minimum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxWidth` | `Int` | — | Sets the maximum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minHeight` | `Int` | — | Sets the minimum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxHeight` | `Int` | — | Sets the maximum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `fill` | `Bool` | `false` | Fills the cross axis in a linear container or both axes in a Stack. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `aspectRatio` | `Float` | — | Sets width divided by height when one dimension determines the other. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `margin` | `Int` | `0` | Sets the outer spacing on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginX` | `Int` | `0` | Sets the outer spacing on the left and right. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginY` | `Int` | `0` | Sets the outer spacing on the top and bottom. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginTop` | `Int` | `0` | Sets the top margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginBottom` | `Int` | `0` | Sets the bottom margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginStart` | `Int` | `0` | Sets the left margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginEnd` | `Int` | `0` | Sets the right margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `padding` | `Int` | `0` | Sets the inset on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingX` | `Int` | `0` | Sets the left and right insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingY` | `Int` | `0` | Sets the top and bottom insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingTop` | `Int` | `0` | Sets the top inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingBottom` | `Int` | `0` | Sets the bottom inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingStart` | `Int` | `0` | Sets the left inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingEnd` | `Int` | `0` | Sets the right inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `alignment` | `Alignment` | `"topStart"` | Positions children along both axes of the stack. | — |

## Scroll

Lays out children linearly and scrolls along its main axis.

**Platform exceptions:** Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux`.

**Children:** Any.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `width` | `Int` | — | Sets the width of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `height` | `Int` | — | Sets the height of the content box. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `grow` | `Float` | `0` | Sets the share of remaining main-axis space for a child in a linear container. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minWidth` | `Int` | — | Sets the minimum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxWidth` | `Int` | — | Sets the maximum content width. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `minHeight` | `Int` | — | Sets the minimum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `maxHeight` | `Int` | — | Sets the maximum content height. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `fill` | `Bool` | `false` | Fills the cross axis in a linear container or both axes in a Stack. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `aspectRatio` | `Float` | — | Sets width divided by height when one dimension determines the other. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `margin` | `Int` | `0` | Sets the outer spacing on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginX` | `Int` | `0` | Sets the outer spacing on the left and right. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginY` | `Int` | `0` | Sets the outer spacing on the top and bottom. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginTop` | `Int` | `0` | Sets the top margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginBottom` | `Int` | `0` | Sets the bottom margin. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginStart` | `Int` | `0` | Sets the left margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `marginEnd` | `Int` | `0` | Sets the right margin in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `padding` | `Int` | `0` | Sets the inset on all four sides. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingX` | `Int` | `0` | Sets the left and right insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingY` | `Int` | `0` | Sets the top and bottom insets. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingTop` | `Int` | `0` | Sets the top inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingBottom` | `Int` | `0` | Sets the bottom inset. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingStart` | `Int` | `0` | Sets the left inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `paddingEnd` | `Int` | `0` | Sets the right inset in left-to-right layout. | Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux` |
| `direction` | `Axis` | `"vertical"` | Selects the scrolling axis at compile time. | — |
| `gap` | `Int` | `0` | Adds space between neighboring children. | — |
| `align` | `CrossAlign` | `"start"` | Positions children on the non-scrolling cross axis. | — |
| `indicators` | `Bool` | `true` | Controls whether scrollbars are shown. | — |

## Spacer

Occupies space on its parent's main axis without rendering content.

**Platform exceptions:** Unsupported: `ios`; Unsupported: `macos`; Unsupported: `android`; Unsupported: `windows`; Unsupported: `linux`.

**Children:** None.

Must be a direct child of: `Column`, `Row`, `Scroll`.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `size` | `Int` | — | Reserves a fixed length; if omitted, shares the remaining space. | — |
