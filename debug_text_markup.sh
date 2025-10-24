#!/bin/bash

set -e

# Build the CLI
echo "Building CLI..."
cargo build --release

CLI="./target/release/re -c openapicli"

# Check if we have the required environment
if [ -z "$REINFER_CLI_TEST_PROJECT" ]; then
    echo "ERROR: REINFER_CLI_TEST_PROJECT not set"
    echo "Need test environment variables. Check if tests can run:"
    echo "  REINFER_CLI_TEST_PROJECT=${REINFER_CLI_TEST_PROJECT}"
    exit 1
fi

# Create a unique source name for testing
SOURCE_NAME="debug-text-markup-$(date +%s)"
FULL_SOURCE="${REINFER_CLI_TEST_PROJECT}/${SOURCE_NAME}"

echo "Using project: ${REINFER_CLI_TEST_PROJECT}"
echo "Creating test source: ${FULL_SOURCE}"

# Try to create source (may fail if auth not set up)
if $CLI create source "${FULL_SOURCE}"; then
    echo "✓ Source created successfully"
else
    echo "✗ Failed to create source - check your CLI authentication"
    exit 1
fi

echo "Uploading comments with text_markup..."
if cat cli/tests/samples/basic.jsonl | $CLI create comments --source="${FULL_SOURCE}" --allow-duplicates --yes; then
    echo "✓ Comments uploaded successfully"
else
    echo "✗ Failed to upload comments"
    $CLI delete source "${FULL_SOURCE}" --yes 2>/dev/null || true
    exit 1
fi

echo "Retrieving comments to check text_markup preservation..."
echo "=== DEBUG OUTPUT (stderr) ==="
$CLI get comments "${FULL_SOURCE}" 2>&1

echo ""
echo "=== CLEANING UP ==="
$CLI delete source "${FULL_SOURCE}"

echo "Done!"
