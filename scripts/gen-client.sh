#!/usr/bin/env bash
set -euo pipefail

# Generate Rust client from a local spec file into ./client (or -o path).
# Usage: scripts/gen-client.sh -i out/reinfer-v1.openapi-v3.1.json [-o client]

GEN_IMG="${GEN_IMG:-openapitools/openapi-generator-cli:v7.8.0}" # using specific version is better than using "latest" 
INPUT=""
OUT_DIR="client"

while [[ $# -gt 0 ]]; do
  case "$1" in
    -i|--input) INPUT="$2"; shift 2;;
    -o|--out) OUT_DIR="$2"; shift 2;;
    -h|--help) echo "Usage: $0 -i <spec.json> [-o <out-dir>]"; exit 0;;
    *) echo "Unknown arg: $1"; exit 2;;
  esac
done

[[ -n "${INPUT:-}" ]] || { echo "✗ Missing -i/--input"; exit 2; }
SPEC_DIR="$(cd "$(dirname "$INPUT")" && pwd)"
SPEC_FILE="$(basename "$INPUT")"

mkdir -p "$OUT_DIR"
rm -rf "$OUT_DIR"/*

echo "▶ Generating Rust client"
echo "   image  : $GEN_IMG"
echo "   input  : $SPEC_DIR/$SPEC_FILE"
echo "   output : $OUT_DIR"

docker run --rm \
  -u "$(id -u):$(id -g)" \
  -v "$PWD:/work" \
  -v "$SPEC_DIR:/spec" \
  -w /work \
  "$GEN_IMG" generate \
    -i "/spec/$SPEC_FILE" \
    -g rust \
    -o "/work/$OUT_DIR" \
    --additional-properties=library=reqwest,supportAsync=false

# Copy patch files (skip if they don't exist)
patches=(
  "name.rs"
  "model_config.rs"  
  "user_properties_value.rs"
  "moon_form_group_update.rs"
  "text_format.rs"
  "inherits_from.rs"
  "message_rich_text_text_markup_inner.rs"
  "markup_table_cell_children_inner.rs"
)

for patch in "${patches[@]}"; do
  if [[ -f "scripts/patches/$patch" ]]; then
    cp "scripts/patches/$patch" "$OUT_DIR/src/models/$patch"
    echo "✓ Applied patch: $patch"
  else
    echo "⚠ Patch file missing (skipping): $patch"
  fi
done

echo "✔ Done → $OUT_DIR"
