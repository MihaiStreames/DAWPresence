#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "Building release binary..."
cargo build --release

BIN_NAME="DAWPresence"
OUT_DIR="$ROOT_DIR/dist"
mkdir -p "$OUT_DIR"

case "$OSTYPE" in
  msys*|cygwin*)
    echo "This script targets Linux/macOS. Use scripts/build.ps1 on Windows."
    exit 1
    ;;
  *)
    cp "target/release/${BIN_NAME}" "$OUT_DIR/"
    echo "Built: $OUT_DIR/${BIN_NAME}"
    ;;
esac
