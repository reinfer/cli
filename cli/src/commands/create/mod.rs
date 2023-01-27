mod annotations;
mod bucket;
mod comments;
mod dataset;
mod emails;
mod project;
mod source;
mod trigger_exception;
mod user;

use self::{
    annotations::CreateAnnotationsArgs, bucket::CreateBucketArgs, comments::CreateCommentsArgs,
    dataset::CreateDatasetArgs, emails::CreateEmailsArgs, project::CreateProjectArgs,
    source::CreateSourceArgs, trigger_exception::CreateTriggerExceptionArgs, user::CreateUserArgs,
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

    #[structopt(name = "trigger-exception")]
    /// Create a new trigger exception
    TriggerException(CreateTriggerExceptionArgs),
}

pub async fn run(create_args: &CreateArgs, client: Client, printer: &Printer) -> Result<()> {
    match create_args {
        CreateArgs::Bucket(bucket_args) => bucket::create(&client, bucket_args, printer).await,
        CreateArgs::Source(source_args) => source::create(&client, source_args, printer).await,
        CreateArgs::Dataset(dataset_args) => dataset::create(&client, dataset_args, printer).await,
        CreateArgs::Project(project_args) => project::create(&client, project_args, printer).await,
        CreateArgs::Comments(comments_args) => comments::create(&client, comments_args).await,
        CreateArgs::Annotations(annotations_args) => {
            annotations::create(&client, annotations_args).await
        }
        CreateArgs::Emails(emails_args) => emails::create(&client, emails_args).await,
        CreateArgs::User(user_args) => user::create(&client, user_args, printer).await,
        CreateArgs::TriggerException(trigger_exception_args) => {
            trigger_exception::create(&client, trigger_exception_args, printer).await
        }
    }
}
