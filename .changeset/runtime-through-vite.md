---
"@weft-ui/vite": minor
---

Apps no longer depend on `@weft-ui/runtime` directly. The plugin resolves every `@weft-ui/runtime` import of a build (generated logic modules, `@weft-ui/platform`, app code) to its own dependency, so a project only installs the plugin, `@weft-ui/platform` and the native shells, and the bundle always uses the runtime whose protocol version was checked against the shell.
