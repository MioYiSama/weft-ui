# Weft UI

A cross-platform UI library.

## Example

```sh
pnpm install
pnpm build:wasm
pnpm --filter example build:<platform>
pnpm test
cargo test --workspace          # compiler, backends, TodoList snapshots
```

After changing `packages/runtime`, run `pnpm build:runtime` to regenerate `packages/miniprogram/runtime`.

Design notes: [docs/PLAN.md](docs/PLAN.md), grammar: [docs/GRAMMAR.md](docs/GRAMMAR.md). Cross-platform roadmap (native views on every platform, JS logic on Hermes): [docs/PLAN-v2.md](docs/PLAN-v2.md).
