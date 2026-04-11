#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
CHANGELOG="${2:-CHANGELOG.md}"

[[ -z "$VERSION" ]] && exit 1
[[ ! -f "$CHANGELOG" ]] && exit 1

# extract section for this version (between this ## and the next ##)
awk -v ver="$VERSION" '
    /^## / {
        if (found) exit
        if (index($0, ver)) { found=1; next }
    }
    found { print }
' "$CHANGELOG" \
| sed '/<p align="right">/,/<\/p>/d' \
| sed -e :a -e '/^[[:space:]]*$/{ $d; N; ba; }' \
| sed '/^[[:space:]]*$/{ N; /^\n$/d; }'
