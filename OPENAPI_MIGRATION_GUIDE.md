# OpenAPI Migration Guide

This guide shows how to migrate from the legacy `reinfer_client` to pure OpenAPI usage in the CLI.

## Complete Migration Mapping

### Core Types

| **Legacy Client** | **OpenAPI Equivalent** | **Notes** |
|-------------------|------------------------|-----------|
| `reinfer_client::Client` | `openapi::apis::configuration::Configuration` | API configuration object |
| `reinfer_client::Token` | Token handled in `Configuration` | Set via `bearer_access_token` |
| `reinfer_client::SourceIdentifier` | `crate::utils::resource_identifier::SourceIdentifier` | Already migrated |
| `reinfer_client::TransformTag` | `crate::utils::transform_tag::TransformTag` | Already migrated |
| `reinfer_client::NewComment` | `openapi::models::CommentNew` | Different name! |
| `reinfer_client::Source` | `openapi::models::Source` | Same structure |

### API Calls

| **Legacy Client** | **OpenAPI Equivalent** | **Usage** |
|-------------------|------------------------|-----------|
| `client.sync_comments()` | `openapi::apis::comments_api::sync_comments()` | Upload comments |
| `client.sync_raw_emails()` | `openapi::apis::emails_api::sync_raw_emails()` | Upload emails |
| `client.get_source()` | `openapi::apis::sources_api::get_source()` | Get source info |
| `client.add_comments()` | `openapi::apis::comments_api::add_comments()` | Add new comments |

## Your New Parser Template

I've created a complete parser template at `/home/edward/reinfer/cli/cli/src/commands/parse/custom_parser.rs` that demonstrates the full OpenAPI migration pattern.

### Key Features

1. **Pure OpenAPI Usage**: No legacy client dependencies
2. **Proper Error Handling**: Uses `anyhow::Context` for error chaining
3. **Progress Tracking**: Integrated with existing progress system
4. **Batch Processing**: Efficient batch uploads
5. **Flexible Parsing**: Easy to adapt to different data formats

### Usage

```bash
# Your new parser is available as:
cargo run -- parse custom --file data.json --source "owner/source-name"

# Or with additional options:
cargo run -- parse custom \
  --file data.json \
  --source "owner/source-name" \
  --no-charge \
  --yes
```

## Key Differences from Legacy Client

### 1. Model Names
- `NewComment` → `CommentNew` 
- API responses use different field names

### 2. Error Handling
```rust
// Legacy
client.sync_comments(source, comments)?;

// OpenAPI
let request = SyncCommentsRequest::new(comments.to_vec());
sync_comments(config, &source.owner, &source.name, request)
    .context("Failed to sync comments")?;
```

### 3. Source Resolution
```rust
// OpenAPI approach
let source = match source_identifier {
    SourceIdentifier::Id(source_id) => {
        let response = get_source_by_id(config, source_id)?;
        response.source
    }
    SourceIdentifier::FullName(full_name) => {
        let response = get_source(config, full_name.owner(), full_name.name())?;
        response.source
    }
};
```

## How to Create Your Own Parser

### 1. Create the Parser File
```rust
// cli/src/commands/parse/your_parser.rs
use openapi::{
    apis::{
        configuration::Configuration,
        comments_api::sync_comments,
        sources_api::{get_source, get_source_by_id},
    },
    models::{CommentNew, Source, SyncCommentsRequest},
};

#[derive(Debug, StructOpt)]
pub struct ParseYourFormatArgs {
    // Your arguments here
}

pub fn parse(config: &Configuration, args: &ParseYourFormatArgs) -> Result<()> {
    // Your parsing logic here
}
```

### 2. Register in mod.rs
```rust
// Add to cli/src/commands/parse/mod.rs
mod your_parser;
use self::your_parser::ParseYourFormatArgs;

// Add to ParseArgs enum
#[derive(Debug, StructOpt)]
pub enum ParseArgs {
    // ... existing parsers ...
    
    #[structopt(name = "your-format")]
    /// Parse your custom format
    YourFormat(ParseYourFormatArgs),
}

// Add to run function
pub fn run(args: &ParseArgs, config: &Configuration, pool: &mut Pool) -> Result<()> {
    match args {
        // ... existing cases ...
        ParseArgs::YourFormat(args) => your_parser::parse(&config, args),
    }
}
```

### 3. Implement Your Parsing Logic

Follow the pattern in `custom_parser.rs`:
1. Define your data structure
2. Parse your format (JSON/CSV/XML/etc.)
3. Convert to `CommentNew`
4. Upload in batches

## Data Format Examples

### JSON Lines
```rust
fn parse_json_line(line: &str) -> Result<CustomRecord> {
    serde_json::from_str(line)
        .with_context(|| "Failed to parse JSON")
}
```

### CSV
```rust
fn parse_csv_file(file_path: &Path) -> Result<Vec<CustomRecord>> {
    let mut reader = csv::Reader::from_path(file_path)?;
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: CustomRecord = result?;
        records.push(record);
    }
    Ok(records)
}
```

### Custom Delimited
```rust
fn parse_delimited(line: &str, delimiter: char) -> Result<CustomRecord> {
    let parts: Vec<&str> = line.split(delimiter).collect();
    if parts.len() < 2 {
        return Err(anyhow!("Invalid format"));
    }
    Ok(CustomRecord {
        id: parts[0].to_string(),
        content: parts[1].to_string(),
        metadata: parts.get(2).map(|s| s.to_string()),
    })
}
```

## Migration Status

### ✅ COMPLETED Migrations
- [x] ✅ `cli/src/main.rs` - **FULLY MIGRATED** (Dec 2024) 🎯 **CRITICAL COMPLETE**
  - Replaced all `reinfer_client` imports with OpenAPI equivalents
  - Created local `Token` wrapper type
  - All commands now use OpenAPI `Configuration` instead of legacy `Client`  
  - Context validation migrated to use OpenAPI patterns
  - Retry logic removed (OpenAPI handles this differently)
  - **Impact**: Core application now fully OpenAPI-based!
- [x] ✅ `cli/src/commands/package/download.rs` - **FULLY MIGRATED** (Dec 2024) 📦 **HIGH PRIORITY COMPLETE** 
  - `AttachmentMetadata` → `openapi::models::Attachment` (with type conversion for size field)
  - `DatasetFlag` → `openapi::models::DatasetFlag` (with `_dataset_flags.contains()` method)
  - `CommentId` → `crate::utils::CommentId` (already existed in utils)
  - `DatasetName` + `.with_project()` → Direct `DatasetFullName` construction
  - `HasAnnotations` → `crate::utils::comment_utils::HasAnnotations`
  - **Impact**: Package download functionality now fully OpenAPI-based!
- [x] ✅ `cli/src/commands/package/upload.rs` - **FULLY MIGRATED** (Dec 2024)
  - All legacy types replaced with OpenAPI equivalents
  - Custom `NewAnnotatedComment` created in utils
  - All compilation errors fixed
- [x] ✅ `cli/src/commands/get/streams.rs` - **FULLY MIGRATED** (Dec 2024)
  - Local wrapper types created for `LabelName` and `ModelVersion`
  - OpenAPI `LabelDef` imported properly
  - All compilation errors fixed

### 🔄 IN PROGRESS / PENDING Migrations

#### High Priority (Command functionality)
- [ ] 📊 `cli/src/commands/get/custom_label_trend_report.rs` - **PARTIALLY MIGRATED**  
  - Legacy imports: `AnnotatedComment`, `Entities`, `LabelName`, `Labelling`
  - Legacy imports: `ModelVersion`, `PredictedLabel`, `Prediction`, `TriggerLabelThreshold`
  - Legacy constant: `DEFAULT_LABEL_GROUP_NAME`
  
#### Lower Priority (Parser/utility functions)
- [ ] 📋 `cli/src/commands/parse/aic_classification_csv.rs`
  - Uses `HasAnnotations` trait from legacy client
  
- [ ] 🏷️ `cli/src/commands/create/annotations.rs`
  - Uses `DEFAULT_LABEL_GROUP_NAME` and potentially other legacy patterns

### Migration Checklist Framework
- [x] ✅ Identified all legacy client usage across codebase
- [x] ✅ Created OpenAPI equivalents mapping  
- [x] ✅ Built parser template with pure OpenAPI
- [x] ✅ Integrated into CLI structure
- [x] ✅ Added proper error handling patterns
- [x] ✅ Added progress tracking
- [x] ✅ Added batch processing
- [x] ✅ Added example data format parsers
- [x] ✅ Successfully migrated 4 critical files (main.rs + 3 command files)
- [x] ✅ **MAJOR MILESTONE**: Core application + key package functionality fully OpenAPI-based

## Next Priority Actions

### 🎉 CRITICAL + HIGH PRIORITY COMPLETE! 
✅ **`main.rs` + `download.rs` FULLY MIGRATED** - Core application + package functionality now 100% OpenAPI-based!

### 📊 NEXT HIGH PRIORITY (Core command functionality)  
1. **Complete `custom_label_trend_report.rs` migration**
   - Replace all legacy types with OpenAPI equivalents or local utils
   - Create wrapper types for `LabelName`, `ModelVersion` if needed (similar to streams.rs)
   - **Impact**: Reporting functionality for customers

### 📋 MEDIUM PRIORITY (Parser/utility cleanup)
2. **Clean up parser files**
   - `aic_classification_csv.rs` - replace `HasAnnotations` usage
   - `annotations.rs` - replace `DEFAULT_LABEL_GROUP_NAME`
   - **Impact**: Less critical but needed for complete migration

## Detailed Migration Steps

### For `main.rs`:
1. Replace `reinfer_client::Client` initialization with OpenAPI `Configuration`
2. Move retry logic to a utility module using OpenAPI patterns
3. Update token handling to use OpenAPI auth
4. Test that all CLI commands still initialize properly

### For `download.rs`:  
1. Import `HasAnnotations` from `crate::utils::comment_utils` 
2. Replace legacy ID types with OpenAPI equivalents
3. Update package writing to use OpenAPI types consistently

### For `custom_label_trend_report.rs`:
1. Create local wrapper types for `LabelName`, `ModelVersion` (reuse from streams.rs)
2. Replace prediction types with OpenAPI equivalents
3. Replace `DEFAULT_LABEL_GROUP_NAME` with a local constant

## Testing Strategy

After each file migration:
1. ✅ Run `cargo check --package reinfer-cli` to verify compilation
2. ✅ Run relevant CLI commands to test functionality  
3. ✅ Update any integration tests that depend on migrated types
4. ✅ Mark migration as complete in this guide

## Original Parser Template Instructions

1. **Test the Template**: Try the custom parser with your data
2. **Adapt for Your Format**: Modify the parsing logic for your specific format
3. **Add Validation**: Add input validation as needed
4. **Add Tests**: Add unit tests for your parsing logic
5. **Optimize**: Add any format-specific optimizations

## Benefits of OpenAPI Migration

1. **No Legacy Dependencies**: Clean, modern code
2. **Better Type Safety**: Generated OpenAPI models are strongly typed
3. **Automatic Updates**: OpenAPI models update with API changes
4. **Better Documentation**: OpenAPI provides self-documenting code
5. **Consistent Patterns**: All new code follows the same pattern

Your custom parser is ready to use and serves as a template for creating additional parsers with pure OpenAPI!
