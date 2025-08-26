#!/usr/bin/env bash
set -euo pipefail

# Fetch a single file from GitHub into the CLI repo.
# Authenticates with SSH sparse-checkout

# Usage:
#   scripts/update-spec.sh --ref <branch|tag|sha> \
#     --repo reinfer/platform \
#     --path backend/api/openapi/reinfer-v1.openapi-v3.1.json \
#     [-o out/reinfer-v1.openapi-v3.1.json]

REF="main"
REPO="reinfer/platform"
REMOTE_PATH="backend/api/openapi/reinfer-v1.openapi-v3.1.json"
OUT="out/reinfer-v1.openapi-v3.1.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ref) REF="$2"; shift 2;;
    --repo) REPO="$2"; shift 2;;
    --path) REMOTE_PATH="$2"; shift 2;;
    -o|--out) OUT="$2"; shift 2;;
    -h|--help)
      echo "Usage: $0 --ref <ref> --repo owner/repo --path path/in/repo [-o out.json]"
      exit 0;;
    *) echo "Unknown arg: $1"; exit 2;;
  esac
done

mkdir -p "$(dirname "$OUT")"

TMP_DIR="$(mktemp -d)"
echo "▶ Sparse-checkout $REPO@$REF:$REMOTE_PATH via SSH"

git clone --filter=blob:none --no-checkout "git@github.com:$REPO.git" "$TMP_DIR" >/dev/null
git -C "$TMP_DIR" sparse-checkout init --no-cone >/dev/null
git -C "$TMP_DIR" sparse-checkout set "$REMOTE_PATH" >/dev/null
git -C "$TMP_DIR" fetch --depth 1 origin "$REF" >/dev/null
git -C "$TMP_DIR" checkout -q FETCH_HEAD

# Copy the file out
mkdir -p "$(dirname "$OUT")"
cp "$TMP_DIR/$REMOTE_PATH" "$OUT"

# Clean up
rm -rf "$TMP_DIR"
echo "✔ Spec updated (SSH) → $OUT"

