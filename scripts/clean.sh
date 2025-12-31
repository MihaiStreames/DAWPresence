#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "Cleaning build artifacts..."
cargo clean

if [[ -d dist ]]; then
  rm -rf dist
  echo "Removed dist/"
fi
