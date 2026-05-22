#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-}"
ANALYSIS="${2:-}"

[[ -z "$TAG" ]] && {
  echo "Usage: $(basename "$0") <tag> <analysis>"
  exit 1
}
[[ -z "$ANALYSIS" ]] && exit 0

BODY=$(gh release view "$TAG" --json body -q .body)

VT_SECTION=""
IFS=',' read -ra ENTRIES <<<"$ANALYSIS"

for entry in "${ENTRIES[@]}"; do
  [[ -z "$entry" ]] && continue

  FILE="${entry%%=*}"
  URL="${entry#*=}"

  BASENAME=$(basename "$FILE")
  VT_SECTION="${VT_SECTION}- [${BASENAME}](${URL})"$'\n'
done

printf '%s\n\n---\n\n### VirusTotal Analysis\n\n%s\n' "$BODY" "$VT_SECTION" >body.md
gh release edit "$TAG" --notes-file body.md
rm -f body.md
