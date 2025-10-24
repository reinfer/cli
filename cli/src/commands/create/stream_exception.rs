//! Commands for creating stream exceptions
//!
//! This module provides functionality to:
//! - Create exceptions for specific comments in streams
//! - Tag comments with exception types for monitoring
//! - Support stream identification via full names

// External crate imports
use anyhow::{Context, Result};
use log::info;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{configuration::Configuration, streams_api::store_exception},
    models::{ExceptionModel, Metadata, StoreExceptionRequest},
};

// Local crate imports
use crate::{printer::Printer, utils::StreamFullName};

/// Command line arguments for creating stream exceptions
#[derive(Debug, StructOpt)]
pub struct CreateStreamExceptionArgs {
    #[structopt(long = "stream")]
    /// The stream full name, qualified by dataset, such as 'my-project-name/my-dataset-name/my-stream-name'.
    stream: StreamFullName,

    #[structopt(long = "type")]
    /// The type of exception. Please choose a short, easy-to-understand string such as "No Prediction".
    r#type: String,

    #[structopt(long = "uid")]
    /// The uid of the comment that should be tagged as an exception.
    uid: String,
}

/// Create a new stream exception for a specific comment
///
/// This function tags a comment with an exception type in the specified stream,
/// which can be used for monitoring and filtering stream processing results.
pub fn create(
    config: &Configuration,
    args: &CreateStreamExceptionArgs,
    _printer: &Printer,
) -> Result<()> {
    let CreateStreamExceptionArgs {
        stream,
        r#type,
        uid,
    } = args;

    create_stream_exception(config, stream, uid, r#type)
        .context("Failed to create stream exception")?;

    info!("New stream exception created successfully");
    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a stream exception for a comment with the specified type
fn create_stream_exception(
    config: &Configuration,
    stream: &StreamFullName,
    uid: &str,
    exception_type: &str,
) -> Result<()> {
    let exception = ExceptionModel {
        uid: uid.to_string(),
        metadata: Box::new(Metadata {
            r#type: exception_type.to_string(),
        }),
    };

    let request = StoreExceptionRequest {
        exceptions: vec![exception],
    };

    store_exception(
        config,
        stream.owner(),
        stream.dataset(),
        stream.stream(),
        request,
    )
    .context("Operation to create a stream exception has failed")?;

    Ok(())
}
