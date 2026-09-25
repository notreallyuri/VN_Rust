#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
public="$root/docs/public"
built="$root/target/wasm32-unknown-unknown/wasm/novn_playground.wasm"

cargo build \
  --manifest-path "$root/Cargo.toml" \
  -p novn-playground \
  --target wasm32-unknown-unknown \
  --profile wasm

mkdir -p "$public/highlight"
cp "$built" "$public/novn-playground.wasm"
cp "$root/docs/src/highlight/tree-sitter-story.wasm" "$public/highlight/"
cp "$root/docs/src/highlight/story.scm" "$public/highlight/"
cp "$root/docs/node_modules/web-tree-sitter/web-tree-sitter.wasm" "$public/highlight/"

size() { echo "$(( $(wc -c < "$1") / 1024 )) KiB"; }
echo "novn-playground.wasm         $(size "$public/novn-playground.wasm")"
echo "highlight/web-tree-sitter    $(size "$public/highlight/web-tree-sitter.wasm")"
echo "highlight/tree-sitter-story  $(size "$public/highlight/tree-sitter-story.wasm")"
