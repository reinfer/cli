# OpenAPI Client Generation Scripts

## Usage

### For Publishing
Use `./cli/publish-binaries` - automatically uses `develop` branch and generates fresh client.

### For Local Development  
Use `./scripts/update-and-generate.sh` - prompts for branch, generates client in `client/` directory.

### For Individual Steps
1. `./scripts/update-spec.sh --ref your-branch` - fetch spec from GitHub
2. `python3 scripts/preprocess-spec.py input.json output.json` - fix type issues  
3. `./scripts/gen-client.sh -i spec.json -o client/` - generate Rust client

## Patches

The following patches are applied because the OpenAPI generator doesn't handle these types correctly:

- `name.rs` - fixes name type generation
- `model_config.rs` - fixes model configuration 
- `user_properties_value.rs` - fixes user property types
- `moon_form_group_update.rs` - fixes moon form handling
- `text_format.rs` - fixes text format types
- `inherits_from.rs` - fixes inheritance types
- `message_rich_text_text_markup_inner.rs` - fixes rich text markup
- `markup_table_cell_children_inner.rs` - fixes table cell types

## Prerequisites

- SSH access to `reinfer/platform` 
- Docker running
- Python 3
