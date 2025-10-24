#!/bin/bash

set -e

CLI="./target/release/re -c openapicli"
SOURCE_NAME="debug-deser-$(date +%s)"
FULL_SOURCE="${REINFER_CLI_TEST_PROJECT}/${SOURCE_NAME}"

echo "Creating source and uploading comments..."
$CLI create source "${FULL_SOURCE}" > /dev/null
cat cli/tests/samples/basic.jsonl | $CLI create comments --source="${FULL_SOURCE}" --allow-duplicates --yes > /dev/null

echo "Getting CLI output and testing deserialization..."
CLI_OUTPUT=$($CLI get comments "${FULL_SOURCE}")

echo "=== CLI JSON OUTPUT ==="
echo "$CLI_OUTPUT"

echo ""
echo "=== TESTING DESERIALIZATION ==="

# Save output to temp file for Rust to read
echo "$CLI_OUTPUT" > /tmp/cli_output.jsonl

cat > /tmp/test_deser.rs << 'EOF'
use serde_json;

#[path = "client/src/models/mod.rs"] 
mod models;
use models::*;

fn main() {
    let input = std::fs::read_to_string("/tmp/cli_output.jsonl").unwrap();
    
    for (line_num, line) in input.lines().enumerate() {
        if line.trim().is_empty() { continue; }
        
        println!("=== LINE {} ===", line_num + 1);
        println!("Raw JSON: {}", line);
        
        match serde_json::from_str::<AnnotatedComment>(line) {
            Ok(annotated_comment) => {
                println!("✓ Successfully parsed as AnnotatedComment");
                
                let comment = &annotated_comment.comment;
                for (msg_idx, message) in comment.messages.iter().enumerate() {
                    println!("  Message {}: text_markup = {:?}", msg_idx + 1, 
                        message.body.text_markup.as_ref().map(|m| m.len()));
                    
                    if let Some(sig) = &message.signature {
                        println!("    Signature: text_markup = {:?}", 
                            sig.text_markup.as_ref().map(|m| m.len()));
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to parse: {}", e);
            }
        }
        println!();
    }
}
EOF

echo "Running Rust deserialization test..."
cd /home/edward/reinfer/cli && rustc --edition 2021 -L target/release/deps /tmp/test_deser.rs -o /tmp/test_deser --extern serde_json=target/release/deps/libserde_json-*.rlib --extern openapi=target/release/deps/libopenapi-*.rlib 2>/dev/null || echo "Could not compile test (dependency issues)"

if [ -f /tmp/test_deser ]; then
    /tmp/test_deser
else
    echo "Skipping deserialization test due to compilation issues"
fi

echo ""
echo "=== CLEANING UP ==="
$CLI delete source "${FULL_SOURCE}" > /dev/null
rm -f /tmp/cli_output.jsonl /tmp/test_deser.rs /tmp/test_deser

echo "Done!"
