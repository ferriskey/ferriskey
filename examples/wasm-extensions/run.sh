#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Building Logger WASM Extension ==="

# Ensure the wasm32-wasip1 target is installed
rustup target add wasm32-wasip1

# Build the logger extension
cargo build \
  --manifest-path "$SCRIPT_DIR/logger/Cargo.toml" \
  --target wasm32-wasip1 \
  --release

# Create an extensions directory and copy the built .wasm file
EXTENSIONS_DIR="$PROJECT_ROOT/extensions"
mkdir -p "$EXTENSIONS_DIR"
cp "$SCRIPT_DIR/logger/target/wasm32-wasip1/release/ferriskey_ext_logger.wasm" "$EXTENSIONS_DIR/"

echo ""
echo "=== Extension built successfully ==="
echo "WASM file: $EXTENSIONS_DIR/ferriskey_ext_logger.wasm"
echo ""
echo "To run FerrisKey with extensions:"
echo "  cargo run --package ferriskey-api -- --extensions-dir $EXTENSIONS_DIR"
