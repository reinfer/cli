//! Create command implementations
//!
//! This module contains all the subcommands for creating various resources
//! in the Reinfer platform, including:
//!
//! - **Buckets**: Email storage containers
//! - **Sources**: Data ingestion endpoints  
//! - **Datasets**: Training data collections
//! - **Projects**: Organizational containers
//! - **Comments**: Text data with annotations
//! - **Annotations**: Labels and entity extraction
//! - **Emails**: Email processing and ingestion
//! - **Users**: User accounts with permissions
//! - **Streams**: Real-time processing pipelines
//! - **Integrations**: External system connections
//! - **Quotas**: Resource usage limits
//! - **Stream Exceptions**: Error handling for streams

// Submodule declarations
pub mod annotations;
pub mod bucket;
pub mod comments;
pub mod dataset;
pub mod emails;
pub mod integrations;
pub mod project;
pub mod quota;
pub mod source;
pub mod stream_exception;
pub mod streams;
pub mod user;

// External crate imports
use anyhow::Result;
use scoped_threadpool::Pool;
use structopt::StructOpt;

// OpenAPI imports
use openapi::apis::configuration::Configuration;

// Local crate imports
use self::{
    annotations::CreateAnnotationsArgs, bucket::CreateBucketArgs, comments::CreateCommentsArgs,
    dataset::CreateDatasetArgs, emails::CreateEmailsArgs, integrations::CreateIntegrationArgs,
    project::CreateProjectArgs, quota::CreateQuotaArgs, source::CreateSourceArgs,
    stream_exception::CreateStreamExceptionArgs, streams::CreateStreamsArgs, user::CreateUserArgs,
};
use crate::printer::Printer;

/// Command line arguments for all create operations
///
/// This enum represents all the different types of resources that can be created
/// through the CLI, with each variant containing the specific arguments needed
/// for that resource type.
#[derive(Debug, StructOpt)]
pub enum CreateArgs {
    #[structopt(name = "bucket")]
    /// Create a new bucket
    Bucket(CreateBucketArgs),

    #[structopt(name = "project")]
    /// Create a new project
    Project(CreateProjectArgs),

    #[structopt(name = "source")]
    /// Create a new source
    Source(CreateSourceArgs),

    #[structopt(name = "dataset")]
    /// Create a new dataset
    Dataset(CreateDatasetArgs),

    #[structopt(name = "comments")]
    /// Create or update comments
    Comments(CreateCommentsArgs),

    #[structopt(name = "annotations")]
    /// Create or update annotations
    Annotations(CreateAnnotationsArgs),

    #[structopt(name = "emails")]
    /// Create or update emails
    Emails(CreateEmailsArgs),

    #[structopt(name = "user")]
    /// Create a new user (note: no welcome email will be sent by default)
    User(CreateUserArgs),

    #[structopt(name = "stream-exception")]
    /// Create a new stream exception
    StreamException(CreateStreamExceptionArgs),

    #[structopt(name = "quota")]
    /// Set a new value for a quota
    Quota(CreateQuotaArgs),

    #[structopt(name = "stream")]
    /// Create a stream
    Stream(CreateStreamsArgs),

    #[structopt(name = "streams")]
    /// Create streams
    Streams(CreateStreamsArgs),

    #[structopt(name = "integration")]
    /// Create integration
    Integration(CreateIntegrationArgs),

    #[structopt(name = "integrations")]
    /// Create integrations
    Integrations(CreateIntegrationArgs),
}

/// Execute the appropriate create command based on the provided arguments
///
/// This function dispatches to the specific create implementation based on the
/// variant of CreateArgs provided. It handles the common setup (configuration,
/// printer, thread pool) and delegates to the specialized create functions.
pub fn run(
    create_args: &CreateArgs,
    config: &Configuration,
    printer: &Printer,
    pool: &mut Pool,
) -> Result<()> {
    match create_args {
        CreateArgs::Bucket(bucket_args) => bucket::create(config, bucket_args, printer),
        CreateArgs::Source(source_args) => source::create(config, source_args, printer),
        CreateArgs::Dataset(dataset_args) => dataset::create(config, dataset_args, printer),
        CreateArgs::Project(project_args) => project::create(config, project_args, printer),
        CreateArgs::Comments(comments_args) => comments::create(config, comments_args, pool),
        CreateArgs::Annotations(annotations_args) => {
            annotations::create(config, annotations_args, pool)
        }
        CreateArgs::Emails(emails_args) => emails::create(config, emails_args),
        CreateArgs::User(user_args) => user::create(config, user_args, printer),
        CreateArgs::StreamException(stream_exception_args) => {
            stream_exception::create(config, stream_exception_args, printer)
        }
        CreateArgs::Quota(quota_args) => quota::create(config, quota_args),
        CreateArgs::Stream(stream_args) | CreateArgs::Streams(stream_args) => {
            streams::create(config, stream_args)
        }
        CreateArgs::Integration(integration_args) | CreateArgs::Integrations(integration_args) => {
            integrations::create(config, integration_args)
        }
    }
}
