#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TARGET="x86_64-pc-windows-gnu"
BUILT_PATH="target/$TARGET/release/DAWPresence.exe"

mkdir -p dist

echo "Building release binary..."
cargo build --release --target "$TARGET"

[[ -f "$BUILT_PATH" ]] || { echo "Build failed: $BUILT_PATH not found" >&2; exit 1; }

cp "$BUILT_PATH" dist/DAWPresence.exe
echo "Built: dist/DAWPresence.exe"
