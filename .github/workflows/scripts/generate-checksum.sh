#!/usr/bin/env bash
set -euo pipefail

DIR="${1:-dist}"

for file in "$DIR"/*.exe; do
  [[ -f "$file" ]] || continue
  HASH=$(sha256sum "$file" | cut -d' ' -f1)
  BASENAME=$(basename "$file")
  printf '%s  %s' "$HASH" "$BASENAME" > "${file}.sha256"
  echo "SHA256: $HASH  $BASENAME"
done
