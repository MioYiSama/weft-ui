# Weft UI example

A small todo app written once and built for all seven platforms: Web, WeChat mini programs, iOS, macOS, Android, Windows and Linux. The [landing page screenshots](https://weftui.mioyi.net/) are this app.

| File | Contents |
| --- | --- |
| [`src/App.weft`](src/App.weft) | The view: models, events and the `App` component |
| [`src/index.ts`](src/index.ts) | The logic: initial model, event handlers, `run(app)` |
| [`vite.config.ts`](vite.config.ts) | The `@weft-ui/vite` plugin with the app ID and name |
| [`index.html`](index.html) | The Web entry |
| `apps/<platform>/` | Platform projects. The templates are committed and belong to the example; `apps/<platform>/weft/` is regenerated on every build, and `apps/web/` is build output only |

## Run it on the Web

From the repository root:

```sh
pnpm install
pnpm build                                              # the compiler and packages, once
pnpm --filter example exec vite                         # dev server
pnpm --filter example build:web                         # or a production build into apps/web/
pnpm --filter example exec vite preview --outDir apps/web
```

## Other platforms

Each native platform needs its kernel built once (see the [root README](../README.md)), then:

```sh
pnpm --filter example build:<platform>   # wechat, macos, ios, android, windows, linux
pnpm --filter example test:<platform>    # UI click flow: macos, ios, android, windows, linux
```

Open `apps/wechat/` in WeChat DevTools. Host requirements per platform are in the [platform guide](https://weftui.mioyi.net/guides/platforms/).

`pnpm --filter example typecheck` type-checks `src/` against the `.weft` declarations through the TypeScript content mapper.

## Learn more

- [Getting started](https://weftui.mioyi.net/guides/getting-started/) walks through this app.
- [Views](https://weftui.mioyi.net/guides/views/) and [Logic and events](https://weftui.mioyi.net/guides/logic/) explain `App.weft` and `index.ts`.
