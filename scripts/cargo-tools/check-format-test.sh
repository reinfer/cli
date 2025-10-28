#!/bin/sh

set -ex

cd "$(dirname "$0")/.."

scripts/cargo-tools/check-dependency-versions.sh
cargo fmt -- --check
cargo clippy --offline --locked --all-targets -- -D warnings
cargo test --offline --locked --all-targets
cargo test --offline --locked --doc
