#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-}"
ANALYSIS="${2:-}"

[[ -z "$TAG" ]] && { echo "Usage: append-virustotal.sh <tag> <analysis>"; exit 1; }
[[ -z "$ANALYSIS" ]] && { echo "No analysis output to append"; exit 0; }

BODY=$(gh release view "$TAG" --json body -q .body)
FILE="${ANALYSIS%%=*}"
URL="${ANALYSIS#*=}"

printf '%s\n\n---\n\n🛡 [VirusTotal Analysis](%s) for `%s`\n' "$BODY" "$URL" "$FILE" > body.md
gh release edit "$TAG" --notes-file body.md
rm -f body.md
