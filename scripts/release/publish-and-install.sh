#!/bin/bash

set -ex

case $BUILD_PLATFORM in
  "ubuntu-24.04")
    rustup target add x86_64-unknown-linux-musl
    rustup target add x86_64-pc-windows-gnu
    pip install toml
    sudo apt-get install          \
      musl-tools                  \
      gcc-mingw-w64-x86-64
    cargo install cargo-deb
    ;;
  "macos-15")
    rustup target add aarch64-apple-darwin
    pip install toml
    brew install mingw-w64
    ;;
  *)
    >&2 echo "fatal: unknown BUILD_PLATFORM '$BUILD_PLATFORM'"
    exit 1
    ;;
esac
