#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p dist

echo "Building release binary..."
cargo build --release --target x86_64-pc-windows-gnu

BUILT_PATH="target/x86_64-pc-windows-gnu/release/DAWPresence.exe"
if [[ ! -f "$BUILT_PATH" ]]; then
    echo "Could not find built binary at: $BUILT_PATH" >&2
    exit 1
fi

cp "$BUILT_PATH" dist/DAWPresence.exe
echo "Built: dist/DAWPresence.exe"
