#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
out="$root/docs/public/novn-playground.wasm"
built="$root/target/wasm32-unknown-unknown/wasm/novn_playground.wasm"

cargo build \
  --manifest-path "$root/Cargo.toml" \
  -p novn-playground \
  --target wasm32-unknown-unknown \
  --profile wasm

mkdir -p "$(dirname -- "$out")"
cp "$built" "$out"
echo "novn-playground.wasm: $(( $(wc -c < "$out") / 1024 )) KiB"
