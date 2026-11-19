# tree-sitter-weft

Tree-sitter grammar for Weft UI `.weft` files, used by the Zed extension (`crates/zed`) for highlighting, brackets, indentation and the outline. The language is specified in `docs/src/content/docs/grammar.md`; the compiler uses a separate parser in `crates/parser`.

```sh
tree-sitter generate   # after editing grammar.js; commit src/ too, editors compile src/parser.c
tree-sitter test       # test/corpus
test/fixtures.sh       # tracked .weft files, except names starting with Broken
```

Zed fetches the grammar from a commit of this repository. After committing grammar changes, set `rev` in `[grammars.weft]` in `crates/zed/extension.toml` to that commit and set `path` to the grammar directory at that commit (`packages/tree-sitter`). Push the grammar commit so Zed can fetch it.
