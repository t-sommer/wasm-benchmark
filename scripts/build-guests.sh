#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="$ROOT/target-guest"

echo "Building guest-core (core wasm)..."
cargo build --release -p guest-core --target wasm32-unknown-unknown

echo "Building guest-component (WIT component)..."
if cargo component build --release -p guest-component; then
  :
else
  echo "cargo component unavailable; using wasm-tools component new"
  cargo build --release -p guest-component --target wasm32-unknown-unknown
  wasm-tools component new \
    "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/guest_component.wasm" \
    -o "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/guest_component.component.wasm"
fi

echo "Done. Artifacts under $CARGO_TARGET_DIR"
