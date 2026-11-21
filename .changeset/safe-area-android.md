---
"@weft-ui/compiler": minor
"@weft-ui/native-android": minor
---

Android apps can set `safeArea: false` to draw behind the system bars and display cutout. The default keeps root content inside the safe area, and `platform.safeArea` still reports the actual insets with either setting.
