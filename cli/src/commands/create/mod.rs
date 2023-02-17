mod annotations;
mod bucket;
mod comments;
mod dataset;
mod emails;
mod project;
mod quota;
mod source;
mod stream_exception;
mod user;

use self::{
    annotations::CreateAnnotationsArgs, bucket::CreateBucketArgs, comments::CreateCommentsArgs,
    dataset::CreateDatasetArgs, emails::CreateEmailsArgs, project::CreateProjectArgs,
    quota::CreateQuotaArgs, source::CreateSourceArgs, stream_exception::CreateStreamExceptionArgs,
    user::CreateUserArgs,
};
use crate::printer::Printer;
use anyhow::Result;
use reinfer_client::Client;
use structopt::StructOpt;

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
}

pub fn run(create_args: &CreateArgs, client: Client, printer: &Printer) -> Result<()> {
    match create_args {
        CreateArgs::Bucket(bucket_args) => bucket::create(&client, bucket_args, printer),
        CreateArgs::Source(source_args) => source::create(&client, source_args, printer),
        CreateArgs::Dataset(dataset_args) => dataset::create(&client, dataset_args, printer),
        CreateArgs::Project(project_args) => project::create(&client, project_args, printer),
        CreateArgs::Comments(comments_args) => comments::create(&client, comments_args),
        CreateArgs::Annotations(annotations_args) => annotations::create(&client, annotations_args),
        CreateArgs::Emails(emails_args) => emails::create(&client, emails_args),
        CreateArgs::User(user_args) => user::create(&client, user_args, printer),
        CreateArgs::StreamException(stream_exception_args) => {
            stream_exception::create(&client, stream_exception_args, printer)
        }
        CreateArgs::Quota(quota_args) => quota::create(&client, quota_args),
    }
}
