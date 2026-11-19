# Changesets

Every user-visible change to a published `@weft-ui/*` package comes with a changeset: run `pnpm changeset`, pick the packages and the bump, and describe the change for the changelog. All published packages share one version (the `fixed` group in `config.json`); before 1.0 a minor bump may break compatibility. `pnpm version-packages` applies the pending changesets to the versions and the `CHANGELOG.md` files. See [protocol/SPEC.md](../protocol/SPEC.md#versioning) for how protocol versions relate to package versions.
