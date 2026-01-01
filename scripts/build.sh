#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BIN_NAME="DAWPresence.exe"
DIST="$ROOT_DIR/dist"
mkdir -p "$DIST"

echo "Building release binary for Windows..."
cargo build --release --target x86_64-pc-windows-gnu

echo "Copying to dist/..."
BUILT_PATH="target/x86_64-pc-windows-gnu/release/$BIN_NAME"

if [[ -f "$BUILT_PATH" ]]; then
    cp "$BUILT_PATH" "$DIST/"
    echo "Built: $DIST/$BIN_NAME"
else
    echo "Could not find built binary at: $BUILT_PATH"
    exit 1
fi
