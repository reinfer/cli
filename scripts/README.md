# Scripts Directory

This directory contains organized scripts for building, testing, and managing the Re:infer CLI project.

## 🚀 For Publishing
Use `./cli/publish-binaries` - automatically uses `develop` branch and generates fresh client.

## 🔧 For Local Development  
Use `./scripts/openapi-client/update-and-generate-complete.sh` - prompts for branch, generates client in `api/` directory.

## ⚙️ For Individual Steps
1. `./scripts/openapi-client/fetch-spec-from-github.sh --ref your-branch` - fetch spec from GitHub
2. `python3 scripts/openapi-client/preprocess-spec.py input.json output.json` - fix type issues  
3. `./scripts/openapi-client/generate-rust-client.sh -i spec.json -o api/` - generate Rust client

---

## 📁 Directory Structure

### 🔧 **cargo-tools/** - Build, Test & Quality Checks
- `build-all-targets.sh` - Build all cargo targets with locked dependencies
- `build-and-install.sh` - Build and install the CLI binary
- `check-format-test.sh` - Run comprehensive checks: format, clippy, tests
- `check-dependency-versions.sh` - Verify cargo dependency versions

### 🌐 **openapi-client/** - API Client Generation  
- `generate-rust-client.sh` - Generate Rust client from OpenAPI spec
- `cleanup-generated-files.sh` - Clean up generated files and fix common issues
- `fetch-spec-from-github.sh` - Download OpenAPI spec from GitHub
- `preprocess-spec.py` - Fix OpenAPI spec type issues before generation
- `update-and-generate-complete.sh` - Complete workflow: fetch, preprocess, generate

### 🚀 **release/** - Publishing & Distribution
- `publish-to-crates-io.sh` - Publish crate to crates.io registry  
- `publish-and-install.sh` - Publish and install the released version

### 🩹 **patches/** - OpenAPI Generator Fixes
Contains Rust code patches applied after generation to fix OpenAPI generator limitations.

## Patches

The following patches are applied because the OpenAPI generator doesn't handle these types correctly:

- **name.rs** - Fixes name type handling
- **model_config.rs** - Configuration model fixes
- **user_properties_value.rs** - User property value type fixes  
- **moon_form_group_update.rs** - Moon form group update fixes
- **text_format.rs** - Text format type fixes
- **inherits_from.rs** - Inheritance relationship fixes
- **message_rich_text_text_markup_inner.rs** - Rich text markup fixes
- **markup_table_cell_children_inner.rs** - Table markup fixes
