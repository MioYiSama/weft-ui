---
"@weft-ui/compiler": minor
---

Add immutable local `let` bindings to Weft. The compiler inlines each binding at its use sites without changing reactive dependencies, and the formatter and editor tooling understand the new syntax.
