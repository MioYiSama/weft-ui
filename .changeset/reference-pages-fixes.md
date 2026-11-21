---
"@weft-ui/compiler": patch
"@weft-ui/runtime": patch
"@weft-ui/native-apple": patch
"@weft-ui/native-android": patch
"@weft-ui/native-windows": patch
"@weft-ui/native-linux": patch
---

Fix layout differences found while building the reference pages: WeChat images no longer let their loaded size override `fill`, stretching or `aspectRatio`; SwiftUI centers button labels in filled or explicitly sized buttons and keeps height-constrained iOS text areas inside their boxes; Android measures custom Button content before Material padding, keeps placeholders in compact inputs and lets `Picker` honor `fill`; WinUI honors explicit input heights without clipping the native border; GTK measures text at its intrinsic width under an at-most constraint and centers custom Button content.
