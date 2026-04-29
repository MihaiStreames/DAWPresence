#!/usr/bin/env bash
set -euo pipefail

CARGO_TOML="${1:-Cargo.toml}"

version=$(grep -m1 '^version' "$CARGO_TOML" | sed 's/.*"\(.*\)".*/\1/')

if [[ -z "$version" ]]; then
  echo "No version found in $CARGO_TOML"
  exit 1
fi

echo "$version"
