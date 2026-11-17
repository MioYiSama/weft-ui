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
```

`WEFT_PLATFORM=<platform> vite build` (comma-separated for several) builds into `apps/<platform>/`. The project template there is copied on the first build and then belongs to you; `apps/<platform>/weft/` is regenerated on every build. Open `apps/wechat/` in WeChat DevTools. Platform APIs come from `@weft-ui/platform` (`run`, `fetch`, `storage`, `platform`, `isSupported`); unsupported ones reject with `PlatformError`. Native platforms (`ios`, `macos`, `android`, `windows`, `linux`) build `apps/<platform>/weft/bundle.hbc` (Hermes bytecode for the kernel in `native/core`) and `layout.json`; their view backends and project templates are not there yet.

Design notes: [docs/PLAN.md](docs/PLAN.md), grammar: [docs/GRAMMAR.md](docs/GRAMMAR.md), view protocol: [docs/PROTOCOL.md](docs/PROTOCOL.md). `WEFT_UPDATE_SNAPSHOTS=1` rewrites Rust snapshots and conformance goldens. Cross-platform roadmap (native views on every platform, JS logic on Hermes): [docs/PLAN-v2.md](docs/PLAN-v2.md).
