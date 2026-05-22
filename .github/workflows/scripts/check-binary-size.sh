#!/usr/bin/env bash
set -euo pipefail

FILE="${1:-}"
MAX_MB="${2:-}"

[[ -z "$FILE" || -z "$MAX_MB" ]] && {
  echo "Usage: $(basename "$0") <file> <max-mb>"
  exit 1
}
[[ ! -f "$FILE" ]] && {
  echo "Error: $FILE not found"
  exit 1
}

SIZE=$(stat -c%s "$FILE" 2>/dev/null || stat -f%z "$FILE")
LIMIT=$((MAX_MB * 1024 * 1024))

if [[ "$SIZE" -gt "$LIMIT" ]]; then
  echo "error: $FILE is ${SIZE} bytes, limit is ${LIMIT} bytes (${MAX_MB}MB)"
  exit 1
fi
