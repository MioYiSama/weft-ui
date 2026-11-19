#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
root=$(git rev-parse --show-toplevel)

tree_sitter=tree-sitter
if [[ -x ./node_modules/.bin/tree-sitter ]]; then
  tree_sitter=./node_modules/.bin/tree-sitter
fi

git -C "$root" ls-files -z -- '*.weft' | {
  fixtures=()
  while IFS= read -r -d '' fixture; do
    [[ ${fixture##*/} == Broken* ]] && continue
    fixtures+=("$root/$fixture")
  done
  if ((${#fixtures[@]} == 0)); then
    echo "No tracked .weft fixtures found" >&2
    exit 1
  fi
  "$tree_sitter" parse --quiet "${fixtures[@]}"
}
