# Weft UI

A cross-platform UI library.

## Example

```sh
pnpm install
pnpm build:wasm
pnpm --filter example build:<platform>
pnpm test                       # includes conformance/: headless ops goldens, Web and mini program parity
cargo test --workspace          # compiler, backends, TodoList snapshots
```

After changing `packages/runtime`, run `pnpm build:runtime` to regenerate `packages/miniprogram/runtime`.

Design notes: [docs/PLAN.md](docs/PLAN.md), grammar: [docs/GRAMMAR.md](docs/GRAMMAR.md), view protocol: [docs/PROTOCOL.md](docs/PROTOCOL.md). `WEFT_UPDATE_SNAPSHOTS=1` rewrites Rust snapshots and conformance goldens. Cross-platform roadmap (native views on every platform, JS logic on Hermes): [docs/PLAN-v2.md](docs/PLAN-v2.md).
