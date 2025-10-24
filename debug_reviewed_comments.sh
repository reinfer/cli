#!/bin/bash
set -e

echo "🔧 Getting current user info..."
USER_OUTPUT=$(re -c openapicli --output=json get current-user)
OWNER=$(echo "edwardclitesting")
echo "✓ Current owner: $OWNER"

echo "🔧 Creating test source..."
TIMESTAMP=$(date +%s)
SOURCE_NAME="$OWNER/debug-source-$TIMESTAMP"
SOURCE_OUTPUT=$(re -c openapicli create source "$SOURCE_NAME")
echo "✓ Created source: $SOURCE_NAME"

echo "🔧 Creating test dataset..."
DATASET_NAME="$OWNER/debug-dataset-$TIMESTAMP"
DATASET_OUTPUT=$(re -c openapicli create dataset "$DATASET_NAME" --source "$SOURCE_NAME")
echo "✓ Created dataset: $DATASET_NAME"

echo "🔧 Uploading test comments..."
re -c openapicli create comments \
  --source "$SOURCE_NAME" \
  --dataset "$DATASET_NAME" \
  --allow-duplicates \
  --yes \
  < cli/tests/samples/many.jsonl
echo "✓ Comments uploaded"

echo "🔧 Getting regular comments first..."
REGULAR_COUNT=$(re -c openapicli get comments "$SOURCE_NAME" | wc -l)
echo "✓ Regular comments count: $REGULAR_COUNT"

echo "🔧 Now trying to get reviewed comments (this is where it might hang)..."
echo "Source name: $SOURCE_NAME"
echo "Dataset name: $DATASET_NAME"

# Add timeout to catch hanging
echo "🔧 Running with 30 second timeout..."
timeout 30s re -c openapicli get comments \
  --reviewed-only true \
  --dataset "$DATASET_NAME" \
  "$SOURCE_NAME" || {
  echo "❌ Command timed out after 30 seconds - it's hanging!"
  echo "🔧 Cleaning up test resources..."
  re -c openapicli delete dataset "$DATASET_NAME" --force >/dev/null 2>&1 || true
  re -c openapicli delete source "$SOURCE_NAME" >/dev/null 2>&1 || true
  echo "✓ Cleanup completed"
  exit 1
}

echo "✓ Reviewed comments completed successfully"

echo "🔧 Cleaning up test resources..."
re -c openapicli delete dataset "$DATASET_NAME" --force >/dev/null 2>&1 || true
re -c openapicli delete source "$SOURCE_NAME" >/dev/null 2>&1 || true
echo "✓ Cleanup completed"
