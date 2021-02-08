mod bucket;
mod comments;
mod dataset;
mod emails;
mod source;
mod user;

use self::{
    bucket::CreateBucketArgs, comments::CreateCommentsArgs, dataset::CreateDatasetArgs,
    emails::CreateEmailsArgs, source::CreateSourceArgs, user::CreateUserArgs,
};
use anyhow::Result;
use reinfer_client::Client;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum CreateArgs {
    #[structopt(name = "bucket")]
    /// Create a new bucket
    Bucket(CreateBucketArgs),

    #[structopt(name = "source")]
    /// Create a new source
    Source(CreateSourceArgs),

    #[structopt(name = "dataset")]
    /// Create a new dataset
    Dataset(CreateDatasetArgs),

    #[structopt(name = "comments")]
    /// Create or update comments
    Comments(CreateCommentsArgs),

    #[structopt(name = "emails")]
    /// Create or update emails
    Emails(CreateEmailsArgs),

    #[structopt(name = "user")]
    /// Create a new user (note: no welcome email will be sent)
    User(CreateUserArgs),
}

pub fn run(create_args: &CreateArgs, client: Client) -> Result<()> {
    match create_args {
        CreateArgs::Bucket(bucket_args) => bucket::create(&client, bucket_args),
        CreateArgs::Source(source_args) => source::create(&client, source_args),
        CreateArgs::Dataset(dataset_args) => dataset::create(&client, dataset_args),
        CreateArgs::Comments(comments_args) => comments::create(&client, comments_args),
        CreateArgs::Emails(emails_args) => emails::create(&client, emails_args),
        CreateArgs::User(user_args) => user::create(&client, user_args),
    }
}
