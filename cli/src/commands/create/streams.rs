//! Commands for creating streams from JSON configuration files
//!
//! This module provides functionality to:
//! - Create multiple streams from JSONL files
//! - Set model versions for stream processing
//! - Handle dataset resolution by name
//! - Process streams with comprehensive error handling

// Standard library imports
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

// External crate imports
use anyhow::{Context, Result};
use log::info;
use serde_json;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{configuration::Configuration, streams_api::create_stream},
    models::{CreateStreamRequest, TriggerNew, TriggerUserModel},
};

// Local crate imports
use crate::utils::types::identifiers::DatasetIdentifier;

/// Command line arguments for creating streams from file
#[derive(Debug, StructOpt)]
pub struct CreateStreamsArgs {
    #[structopt(short = "d", long = "dataset")]
    /// Dataset where the streams should be created
    dataset_id: DatasetIdentifier,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with streams
    path: PathBuf,

    #[structopt(short = "v", long = "model-version")]
    /// The model version for the new streams to use
    model_version: i32,
}

/// Create multiple streams from a JSONL configuration file
///
/// This function handles:
/// - Dataset resolution by full name
/// - Reading stream configurations from JSONL files
/// - Setting model versions for all streams
/// - Individual stream creation with error reporting
pub fn create(config: &Configuration, args: &CreateStreamsArgs) -> Result<()> {
    let CreateStreamsArgs {
        path,
        dataset_id,
        model_version,
    } = args;

    let file =
        File::open(path).with_context(|| format!("Could not open file `{}`", path.display()))?;
    let file = BufReader::new(file);
    let dataset = crate::utils::resolve_dataset(config, dataset_id)?;

    for read_stream_result in read_streams_iter(file) {
        let trigger_new = read_stream_result?;
        create_single_stream(config, &dataset, trigger_new, *model_version)
            .context("Failed to create stream")?;
    }
    Ok(())
}

/// Create an iterator that reads and parses stream configurations from JSONL
fn read_streams_iter<'a>(
    mut streams: impl BufRead + 'a,
) -> impl Iterator<Item = Result<TriggerNew>> + 'a {
    let mut line = String::new();
    let mut line_number: u32 = 0;
    std::iter::from_fn(move || {
        line_number += 1;
        line.clear();

        let read_result = streams
            .read_line(&mut line)
            .with_context(|| format!("Could not read line {line_number} from input stream"));

        match read_result {
            Ok(0) => return None,
            Err(e) => return Some(Err(e)),
            _ => {}
        }

        Some(
            serde_json::from_str::<TriggerNew>(line.trim_end()).with_context(|| {
                format!("Could not parse stream configuration at line {line_number}")
            }),
        )
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a single stream with the specified model version
fn create_single_stream(
    config: &Configuration,
    dataset: &openapi::models::Dataset,
    mut trigger_new: TriggerNew,
    model_version: i32,
) -> Result<()> {
    // Set the model version
    let trigger_user_model = TriggerUserModel::new(model_version);
    trigger_new.model = Some(Some(Box::new(trigger_user_model)));

    let request = CreateStreamRequest {
        stream: Box::new(trigger_new.clone()),
    };

    create_stream(config, &dataset.owner, &dataset.name, request)
        .context("Failed to create stream via API")?;

    let stream_name = trigger_new.name.as_deref().unwrap_or("unnamed");
    info!(
        "Created stream '{}'in dataset '{}/{}'",
        stream_name, dataset.owner, dataset.name
    );

    Ok(())
}
