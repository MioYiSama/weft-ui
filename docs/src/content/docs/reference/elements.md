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
| `padding` | `Int` | — | Sets the same inset on all four sides. | — |
| `gap` | `Int` | — | Adds space between neighboring children. | — |
| `width` | `Int` | — | Sets a fixed width instead of sizing to the content. | — |
| `height` | `Int` | — | Sets a fixed height instead of sizing to the content. | — |
| `background` | `Color` | — | Sets the background color. | — |

## Row

Places children side by side.

**Children:** Any.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `padding` | `Int` | — | Sets the same inset on all four sides. | — |
| `gap` | `Int` | — | Adds space between neighboring children. | — |
| `width` | `Int` | — | Sets a fixed width instead of sizing to the content. | — |
| `height` | `Int` | — | Sets a fixed height instead of sizing to the content. | — |
| `background` | `Color` | — | Sets the background color. | — |

## Text

Displays text.

**Children:** None.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `value` (positional) | `String` | Required | The text to display. | — |
| `color` | `Color` | — | Sets the text color. | — |
| `size` | `Int` | — | Sets the font size in logical pixels. | — |
| `bold` | `Bool` | — | Displays the text in bold. | — |

## Button

Renders a clickable control with child nodes for its content.

**Children:** Any.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
| `disabled` | `Bool` | — | Disables the button. | — |

| Event | Payload | Description | Platform exceptions |
| --- | --- | --- | --- |
| `onClick` | — | Fires when the button is clicked. | — |

## Input

Displays an editable text field.

**Children:** None.

| Property | Type | Default | Description | Platform exceptions |
| --- | --- | --- | --- | --- |
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
| `src` (positional) | `String` | Required | The image URL. | — |
| `width` | `Int` | — | Sets the image width. | — |
| `height` | `Int` | — | Sets the image height. | — |
