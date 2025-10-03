# OpenAPI Spec Update and Client Generation Guide

This guide walks you through the process of updating the OpenAPI specification, preprocessing it, and generating the Rust client code based on your latest branch commit.

## Current Branch Context

**Branch**: `edward/rust-client`

## Overview

The process consists of three main steps:
1. **Update Spec**: Fetch the latest OpenAPI specification from the platform repository
2. **Preprocess Spec**: Fix type issues and validate the specification 
3. **Generate Client**: Create the Rust client code from the processed specification

## Prerequisites

- SSH access to `reinfer/platform` repository
- Docker installed (for client generation)
- Python 3 (for preprocessing)
- Bash shell

## Step 1: Update Spec

Fetch the latest OpenAPI specification from the platform repository:

```bash
# Update from your current branch (edward/rust-client)
./scripts/update-spec.sh --ref edward/rust-client \
  --repo reinfer/platform \
  --path backend/api/openapi/reinfer-v1.openapi-v3.1.json \
  -o out/reinfer-v1.openapi-v3.1.json
```

### Parameters:
- `--ref`: Git reference (branch, tag, or SHA) - use your branch name
- `--repo`: Source repository in format `owner/repo`
- `--path`: Path to the OpenAPI spec file in the source repo
- `-o, --out`: Output file path (optional, defaults to `out/reinfer-v1.openapi-v3.1.json`)

### What it does:
- Creates a sparse checkout of the specified repository
- Fetches only the OpenAPI spec file using SSH
- Copies the file to your local `out/` directory
- Cleans up temporary files

## Step 2: Preprocess Spec

Fix known type issues in the OpenAPI specification:

```bash
python3 scripts/preprocess-spec.py \
  out/reinfer-v1.openapi-v3.1.json \
  out/reinfer-v1.openapi-v3.1.json
```

### What it fixes:
- **EntityDefNew.id**: Converts literal `None` types to nullable strings
- **FieldChoiceNewApi.id**: Converts literal `None` types to nullable strings  
- **General validation**: Scans for and fixes other invalid schema types
- **Schema validation**: Validates the output to ensure it's valid for code generation

### Output:
- ✓ Reports fixed schema issues
- ✓ Validates the final specification
- ✗ Reports any remaining issues that need manual intervention

## Step 3: Generate Client

Generate the Rust client code using OpenAPI Generator:

```bash
./scripts/gen-client.sh -i out/reinfer-v1.openapi-v3.1.json -o out/rust
```

### Parameters:
- `-i, --input`: Path to the preprocessed OpenAPI spec file
- `-o, --out`: Output directory for generated client (optional, defaults to `out/rust`)

### What it does:
- Uses Docker with `openapitools/openapi-generator-cli:v7.8.0`
- Generates Rust client using the `reqwest` library
- Configures for synchronous operations (`supportAsync=false`)
- Clears existing generated code before creating new client
- Preserves file ownership using your user/group ID

### Generated Structure:
```
out/rust/
├── Cargo.toml
├── README.md
├── docs/           # API documentation
├── git_push.sh     # Helper script
└── src/
    ├── apis/       # API endpoint implementations
    ├── models/     # Data models
    └── lib.rs      # Library entry point
```

## Complete Example

Here's the full process for updating from your current branch:

```bash
# 1. Update spec from your branch
./scripts/update-spec.sh --ref edward/rust-client \
  --repo reinfer/platform \
  --path backend/api/openapi/reinfer-v1.openapi-v3.1.json

# 2. Preprocess to fix type issues
python3 scripts/preprocess-spec.py \
  out/reinfer-v1.openapi-v3.1.json \
  out/reinfer-v1.openapi-v3.1.json

# 3. Generate Rust client
./scripts/gen-client.sh -i out/reinfer-v1.openapi-v3.1.json

echo "✔ Client generation complete! Check out/rust/ for the generated code."
```

## Troubleshooting

### SSH Authentication Issues
- Ensure you have SSH keys set up for GitHub
- Test with: `ssh -T git@github.com`

### Docker Permission Issues
- Ensure Docker is running and your user has access
- The script uses `$(id -u):$(id -g)` to preserve file ownership

### Preprocessing Failures
- Check the Python script output for specific schema issues
- Manual intervention may be required for complex type problems

### Generation Failures
- Ensure the input JSON file is valid
- Check Docker logs if the container fails
- Verify the OpenAPI spec version compatibility

## Future Automation

**This entire process will be automated by CI/CD pipeline in the future.**

The planned automation will:
- **Trigger**: Automatically detect changes in the platform repository's OpenAPI spec
- **Update**: Fetch the latest spec from the specified branch
- **Process**: Run preprocessing and validation automatically  
- **Generate**: Create updated client code
- **Test**: Run automated tests against the new client
- **Deploy**: Publish updated client packages
- **Notify**: Alert developers of successful updates or failures

### Pipeline Benefits:
- 🔄 **Continuous sync** with platform API changes
- 🛡️ **Automated validation** prevents broken client code
- 📦 **Consistent releases** with proper versioning
- 🚀 **Faster iteration** without manual intervention
- 🔍 **Better tracking** of API changes and client updates

Until the pipeline is implemented, use this manual process to keep your client code up to date with the latest API specifications from your development branch.

## Related Files

- `scripts/update-spec.sh`: Spec fetching script
- `scripts/preprocess-spec.py`: Spec preprocessing and validation
- `scripts/gen-client.sh`: Client code generation
- `out/reinfer-v1.openapi-v3.1.json`: OpenAPI specification
- `out/rust/`: Generated Rust client code
- `.github/workflows/`: Future automation pipeline definitions
