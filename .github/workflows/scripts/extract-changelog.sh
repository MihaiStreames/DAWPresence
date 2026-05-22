#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
CHANGELOG="${2:-}"

[[ -z "$VERSION" || -z "$CHANGELOG" ]] && {
  echo "Usage: $(basename "$0") <version> <changelog>"
  exit 1
}
[[ ! -f "$CHANGELOG" ]] && {
  echo "Error: $CHANGELOG not found"
  exit 1
}

VERSION="${VERSION#v}"

section="$(awk -v ver="$VERSION" '
  /^## \[/ {
    if (found) exit
    if (index($0, "[" ver "]")) { found=1; next }
  }
  found { print }
' "$CHANGELOG" | sed -e :a -e '/^[[:space:]]*$/{ $d; N; ba; }')"

if [[ -z "$section" ]]; then
  echo "Failed to extract release notes for '$VERSION' from $CHANGELOG" >&2
  exit 1
fi

printf '%s\n' "$section"
