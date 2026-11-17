# Weft UI

A cross-platform UI library.

## Web demo

```sh
pnpm install
pnpm build:wasm                 # compiler as WASM for the Vite plugin
pnpm --filter todo-web dev      # examples/todo-web
pnpm test                       # end-to-end tests (happy-dom)
```

## Rust demo

```sh
cargo run -p todo-rust          # examples/todo-rust in a native window (WebView)
cargo test --workspace --all-features
pnpm build:runtime              # after changing packages/runtime: regenerate src/js and packages/miniprogram/runtime
```

## WeChat mini program demo

```sh
pnpm build:wasm
pnpm --filter todo-wx build     # compiles ui/TodoList.weft into miniprogram/weft
```

Open `examples/todo-wx` in WeChat DevTools. To use your own AppID, put it in `project.private.config.json`. Tests (`pnpm test`) check the exact `setData` payloads in Node.

Design notes: [docs/PLAN.md](docs/PLAN.md), grammar: [docs/GRAMMAR.md](docs/GRAMMAR.md).
