#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "Usage: $(basename "$0") <file> [file...]"
  exit 1
fi

for f in "$@"; do
  sha256sum "$f" | awk '{print $1}' > "${f}.sha256"
  echo "${f}.sha256"
done
