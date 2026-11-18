# Weft UI

A cross-platform UI library.

## Example

```sh
pnpm install
pnpm build:wasm
pnpm --filter example build:web      # Vite app in example/apps/web/
pnpm --filter example build:wechat   # mini program project in example/apps/wechat/
pnpm test                       # includes conformance/: headless ops goldens, Web and mini program parity
cargo test --workspace          # compiler, backends, TodoList snapshots

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

`WEFT_PLATFORM=<platform> vite build` (comma-separated for several) builds into `apps/<platform>/`. The project template there is copied on the first build and then belongs to you; `apps/<platform>/weft/` is regenerated on every build. Open `apps/wechat/` in WeChat DevTools. Platform APIs come from `@weft-ui/platform` (`run`, `fetch`, `storage`, `platform`, `isSupported`); unsupported ones reject with `PlatformError`. Native platforms (`ios`, `macos`, `android`, `windows`, `linux`) build `apps/<platform>/weft/bundle.hbc` (Hermes bytecode for the kernel in `native/core`) and `layout.json`. `macos` and `ios` also generate SwiftUI views into the `WeftApp` package in `apps/<platform>/weft/`, linked by the Xcode project `apps/<platform>/App.xcodeproj` and its WeftKit shell (`native/apple`), and run `xcodebuild` (`WEFT_NATIVE=0` only generates the project). `windows` generates WinUI 3 views (`weft/Generated.cs`) for the dotnet project `apps/windows/App.csproj`, linked against the Weft.WinUI shell (`native/windows`), and runs `dotnet build` on Windows hosts. `android` generates Jetpack Compose views into the Gradle module `apps/android/weft/` (with the bytecode as an asset), used by the Gradle project `apps/android/` together with its Android shell (`native/android`), and runs `./gradlew assembleDebug` on any host. `linux` generates GTK 4 views (`weft/Generated.cpp`) for the CMake project `apps/linux/`, linked against the C++ shell (`native/linux`), and builds it with CMake on Linux hosts (the executable is `apps/linux/build/app`).

Design notes: [docs/PLAN.md](docs/PLAN.md), grammar: [docs/GRAMMAR.md](docs/GRAMMAR.md), view protocol: [docs/PROTOCOL.md](docs/PROTOCOL.md). `WEFT_UPDATE_SNAPSHOTS=1` rewrites Rust snapshots and conformance goldens. Cross-platform roadmap (native views on every platform, JS logic on Hermes): [docs/PLAN-v2.md](docs/PLAN-v2.md).
