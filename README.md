# Weft UI

A cross-platform UI library.

## Example

```sh
pnpm install
pnpm build:wasm
pnpm --filter example build:web      # Vite app in example/apps/web/
pnpm --filter example build:wechat   # mini program project in example/apps/wechat/
pnpm test                       # includes tests/conformance/: headless ops goldens, Web and mini program parity
cargo test --workspace          # parser, transformer backends, TodoList snapshots

# Native kernel (Hermes is fetched and built from source on the first configure)
cmake -S native/core -B native/core/build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build native/core/build
ctest --test-dir native/core/build   # kernel tests + conformance fixtures in Hermes

# Apple (macOS host with Xcode): the kernel as WeftCore.xcframework, once
native/apple/build-core.sh
pnpm --filter example build:macos    # Xcode project in example/apps/macos/, built with xcodebuild
pnpm --filter example build:ios      # same for the iOS simulator
pnpm --filter example test:macos     # XCUITest click flow (test:ios for the simulator)

# Android (any host with the Android SDK + NDK, CMake, Ninja, Python 3, JDK 17+): the kernel as libweft.so, once
native/android/build-core.sh         # WEFT_ANDROID_ABIS="arm64-v8a x86_64" for a subset
pnpm --filter example build:android  # Gradle project in example/apps/android/, built with its wrapper
pnpm --filter example test:android   # Compose UI Test click flow on a connected device or emulator

# Windows (Windows host with VS Build Tools C++ + CMake, Python 3, .NET SDK 10 or later): the kernel as weft_core.dll, once
native/windows/build-core.ps1        # -Arch arm64|x64, default: the host's
pnpm --filter example build:windows  # dotnet project in example/apps/windows/, built with dotnet build
pnpm --filter example test:windows   # UI Automation (FlaUI) click flow; needs an interactive desktop
dotnet test native/windows/Weft.WinUI.Tests

# Linux (Linux host with a C++20 compiler, CMake, Ninja, Python 3, ICU and GTK 4 / libsoup 3 / json-glib dev packages): the kernel as libweft_core.so, once
native/linux/build-core.sh           # also builds hermesc, which hermes-compiler lacks for arm64
pnpm --filter example build:linux    # CMake project in example/apps/linux/, built into apps/linux/build/
pnpm --filter example test:linux     # in-process click flow; needs a display
# or all of it in Docker, with an XFCE desktop on vnc://localhost:5901 (password weft)
docker compose -f native/linux/docker/compose.yaml up -d --build
docker compose -f native/linux/docker/compose.yaml exec linux bash
```

## Editor support

```sh
cargo install --path crates/cli      # installs `weft`
weft fmt [--check] [paths...]        # format .weft files (default: current directory, honoring .gitignore)
weft lsp                             # language server on stdio: diagnostics, completion, hover, go to definition, formatting
```

Zed: run `zed: install dev extension` and pick `crates/zed/` (needs Rust installed through rustup). The extension starts `weft lsp` from `PATH`; set `lsp.weft.binary.path` in Zed's settings to use another binary. Highlighting, brackets, indentation and the outline come from the tree-sitter grammar in `packages/tree-sitter/`. Zed fetches the grammar from a pinned commit; see [how to update the pin](packages/tree-sitter/README.md).

## TypeScript

Types of `.weft` imports come from the [TypeScript 7.1 content mapper](https://devblogs.microsoft.com/typescript/announcing-typescript-7-1-beta/#content-mappers) `@weft-ui/content-mapper` (after `pnpm build:wasm`). Add it as a dev dependency and register it in `tsconfig.json`, as `example/tsconfig.json` does:

```jsonc
"contentMappers": [{ "package": "@weft-ui/content-mapper", "extensions": [".weft"] }]
```

Then run `tsc --runExternalCode` (`pnpm --filter example typecheck`); VS Code starts the mapper in trusted workspaces. TypeScript sees each `.weft` file as the declarations of its logic module: Weft errors are reported in the `.weft` file, and the names of models, fields, events, components and parameters map back to their declarations, so type errors, go to definition and find references land in the `.weft` source. Without content mappers, `dts: true` makes the Vite plugin write `X.d.weft.ts` next to every `X.weft` (needs `allowArbitraryExtensions`).

## Vite plugin

With `@weft-ui/vite`, `entry` selects the JS/TS entry for non-Web builds, relative to the Vite root. It defaults to the first existing file in `src/index.{js,ts,mjs,mts,cjs,cts}`, in that order. Web builds start from `index.html`. Set `apps` to change the output directory for platform projects (an absolute path or a path relative to the Vite root; default `./apps`). The paths below assume the default.

Set `native` to override the directory containing the native shells (`apple/`, `android/`, `windows/`, `linux/`). It accepts an absolute path or a path relative to the Vite root. By default, the plugin uses the Weft workspace's `native/` directory.

`WEFT_PLATFORM=<platform> vite build` (comma-separated for several) builds into `apps/<platform>/`. The project template there is copied on the first build and then belongs to you; `apps/<platform>/weft/` is regenerated on every build. Open `apps/wechat/` in WeChat DevTools. Platform APIs come from `@weft-ui/platform` (`run`, `fetch`, `storage`, `platform`, `isSupported`); unsupported ones reject with `PlatformError`. Native platforms (`ios`, `macos`, `android`, `windows`, `linux`) build `apps/<platform>/weft/bundle.hbc` (Hermes bytecode for the kernel in `native/core`) and `layout.json`. `macos` and `ios` also generate SwiftUI views into the `WeftApp` package in `apps/<platform>/weft/`, linked by the Xcode project `apps/<platform>/App.xcodeproj` and its WeftKit shell (`native/apple`), and run `xcodebuild` (`WEFT_NATIVE=0` only generates the project). `windows` generates WinUI 3 views (`weft/Generated.cs`) for the dotnet project `apps/windows/App.csproj`, linked against the Weft.WinUI shell (`native/windows`), and runs `dotnet build` on Windows hosts. `android` generates Jetpack Compose views into the Gradle module `apps/android/weft/` (with the bytecode as an asset), used by the Gradle project `apps/android/` together with its Android shell (`native/android`), and runs `./gradlew assembleDebug` on any host. `linux` generates GTK 4 views (`weft/Generated.cpp`) for the CMake project `apps/linux/`, linked against the C++ shell (`native/linux`), and builds it with CMake on Linux hosts (the executable is `apps/linux/build/app`).

`WEFT_UPDATE_SNAPSHOTS=1` rewrites Rust snapshots (`crates/transformer/tests/snapshots/`) and conformance goldens (`tests/conformance/golden/`).

## Docs

The documentation site lives in `docs/` (Astro + Starlight) and is published to <https://weftui.mioyi.net/> (GitHub Pages) by `.github/workflows/docs.yml` on every push to `master` that touches it. English is the root locale, Chinese lives under `/zh/`; a first visit follows the browser language. Grammar: [docs/src/content/docs/grammar.md](docs/src/content/docs/grammar.md) ([中文](docs/src/content/docs/zh/grammar.md)), view protocol: [docs/src/content/docs/protocol.md](docs/src/content/docs/protocol.md) ([中文](docs/src/content/docs/zh/protocol.md)). Landing page copy: `docs/src/i18n/landing.ts`.

```sh
pnpm --filter docs dev     # http://localhost:4321/
pnpm --filter docs build   # static site in docs/dist/
```
