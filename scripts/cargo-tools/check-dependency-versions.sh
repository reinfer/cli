#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")/../.."

eprintln() {
  local message="$1"
  >&2 echo "$message"
}

VERSION=$(grep -e "^version" cli/Cargo.toml | cut -d ' ' -f 3 | tr -d '"')
README="README.md"
eprintln "Enforcing links for version '$VERSION'"

expect_link() {
  local link="$1"
  eprintln "Enforcing link '$link' in '$README'"
  if ! grep -Fq "$link" "$README"; then
      eprintln "error: '$README' does not contain expected link '$link', exiting"
      exit 1;
  fi
}

expect_link "https://reinfer.dev/public/cli/bin/x86_64-unknown-linux-musl/$VERSION/re"
expect_link "https://reinfer.dev/public/cli/bin/aarch64-apple-darwin/$VERSION/re"
expect_link "https://reinfer.dev/public/cli/bin/x86_64-pc-windows-gnu/$VERSION/re.exe"
expect_link "https://reinfer.dev/public/cli/debian/reinfer-cli_${VERSION}_amd64.deb"
