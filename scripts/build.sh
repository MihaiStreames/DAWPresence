#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BIN_NAME="DAWPresence.exe"
OUT_DIR="$ROOT_DIR/dist"
mkdir -p "$OUT_DIR"

echo "Building release binary..."
cargo build --release

echo "Copying to dist/..."
cp "target/x86_64-pc-windows-gnu/release/${BIN_NAME}" "$OUT_DIR/"
echo "Built: $OUT_DIR/${BIN_NAME}"
