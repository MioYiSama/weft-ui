# tree-sitter-weft

Tree-sitter grammar for Weft UI `.weft` files, used by the Zed extension (`editors/zed`) for highlighting, brackets, indentation and the outline. The language itself is specified in `docs/GRAMMAR.md`; the compiler has its own parser.

```sh
tree-sitter generate   # after editing grammar.js; commit src/ too, editors compile src/parser.c
tree-sitter test       # test/corpus
```

Zed fetches the grammar from a commit of this repository: after committing grammar changes, set `rev` in `editors/zed/extension.toml` (`[grammars.weft]`) to that commit.
