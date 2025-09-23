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

## Migration Checklist

- [x] ✅ Identified all legacy client usage
- [x] ✅ Created OpenAPI equivalents mapping
- [x] ✅ Built parser template with pure OpenAPI
- [x] ✅ Integrated into CLI structure
- [x] ✅ Added proper error handling
- [x] ✅ Added progress tracking
- [x] ✅ Added batch processing
- [x] ✅ Added example data format parsers
- [x] ✅ Fixed all linter errors

## Next Steps

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
