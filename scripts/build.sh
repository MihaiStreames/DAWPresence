#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BIN_NAME="DAWPresence.exe"
DIST="$ROOT_DIR/dist"
mkdir -p "$DIST"

echo "Building release binary..."
cargo build --release

echo "Copying to dist/..."
POSSIBLE_PATHS=(
    "target/release/$BIN_NAME"
    "target/x86_64-pc-windows-gnu/release/$BIN_NAME"
    "target/x86_64-pc-windows-msvc/release/$BIN_NAME"
)

FOUND=""
for path in "${POSSIBLE_PATHS[@]}"; do
    if [[ -f "$path" ]]; then
        FOUND="$path"
        break
    fi
done

if [[ -n "$FOUND" ]]; then
    cp "$FOUND" "$DIST/"
    echo "Built: $DIST/$BIN_NAME"
else
    echo "Could not find built binary in any of: ${POSSIBLE_PATHS[*]}"
    exit 1
fi
