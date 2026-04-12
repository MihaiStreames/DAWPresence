#!/usr/bin/env bash
set -euo pipefail

check_size() {
    local file="$1"
    local max_mb="$2"
    local size limit

    size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file")
    limit=$((max_mb * 1024 * 1024))

    echo "$file: ${size} bytes (limit: ${limit} bytes)"

    if [[ "$size" -gt "$limit" ]]; then
        echo "FAIL: $file too large"
        return 1
    fi
}

check_size "dist/DAWPresence-tiny-skia.exe" 4
check_size "dist/DAWPresence-wgpu.exe" 5
