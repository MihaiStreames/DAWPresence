#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-}"
ANALYSIS="${2:-}"

[[ -z "$TAG" ]] && { echo "Usage: append-virustotal.sh <tag> <analysis>"; exit 1; }
[[ -z "$ANALYSIS" ]] && { echo "No analysis output to append"; exit 0; }

BODY=$(gh release view "$TAG" --json body -q .body)

VT_SECTION=""
while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    FILE="${line%%=*}"
    URL="${line#*=}"
    BASENAME=$(basename "$FILE")
    VT_SECTION="${VT_SECTION}- [${BASENAME}](${URL})\n"
done <<< "$ANALYSIS"

printf '%s\n\n---\n\n### VirusTotal Analysis\n\n%b\n' "$BODY" "$VT_SECTION" > body.md
gh release edit "$TAG" --notes-file body.md
rm -f body.md
