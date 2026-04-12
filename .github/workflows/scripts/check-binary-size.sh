#!/usr/bin/env bash
set -euo pipefail

FILE="${1:-dist/DAWPresence.exe}"
MAX_MB="${2:-4}"

SIZE=$(stat -c%s "$FILE" 2>/dev/null || stat -f%z "$FILE")
LIMIT=$((MAX_MB * 1024 * 1024))

echo "Binary size: ${SIZE} bytes (limit: ${LIMIT} bytes)"

if [[ "$SIZE" -gt "$LIMIT" ]]; then
    echo "Binary too large: ${SIZE} bytes (limit: ${LIMIT} bytes)"
    exit 1
fi
