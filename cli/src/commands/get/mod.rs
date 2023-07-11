mod buckets;
mod comments;
mod datasets;
mod projects;
mod quota;
mod sources;
mod streams;
mod users;

use anyhow::Result;
use reinfer_client::Client;
use structopt::StructOpt;

use self::{
    buckets::GetBucketsArgs,
    comments::{GetManyCommentsArgs, GetSingleCommentArgs},
    datasets::GetDatasetsArgs,
    projects::GetProjectsArgs,
    sources::GetSourcesArgs,
    streams::{GetStreamCommentsArgs, GetStreamsArgs},
    users::GetUsersArgs,
};
use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub enum GetArgs {
    #[structopt(name = "buckets")]
    /// List the available buckets
    Buckets(GetBucketsArgs),

    #[structopt(name = "comment")]
    /// Get a single comment from a source
    Comment(GetSingleCommentArgs),

    #[structopt(name = "comments")]
    /// Download all comments from a source
    Comments(GetManyCommentsArgs),

    #[structopt(name = "datasets")]
    /// List the available datasets
    Datasets(GetDatasetsArgs),

    #[structopt(name = "projects")]
    /// List the available projects
    Projects(GetProjectsArgs),

    #[structopt(name = "sources")]
    /// List the available sources
    Sources(GetSourcesArgs),

    #[structopt(name = "streams")]
    /// List the available streams for a dataset
    Streams(GetStreamsArgs),

    #[structopt(name = "stream-comments")]
    /// Fetch comments from a stream
    StreamComments(GetStreamCommentsArgs),

    #[structopt(name = "users")]
    /// List the available users
    Users(GetUsersArgs),

    #[structopt(name = "current-user")]
    /// Get the user associated with the API token in use
    CurrentUser,

    #[structopt(name = "quotas")]
    /// List all quotas for current tenant
    Quotas,
}

pub fn run(args: &GetArgs, client: Client, printer: &Printer) -> Result<()> {
    match args {
        GetArgs::Buckets(args) => buckets::get(&client, args, printer),
        GetArgs::Comment(args) => comments::get_single(&client, args),
        GetArgs::Comments(args) => comments::get_many(&client, args),
        GetArgs::Datasets(args) => datasets::get(&client, args, printer),
        GetArgs::Projects(args) => projects::get(&client, args, printer),
        GetArgs::Sources(args) => sources::get(&client, args, printer),
        GetArgs::Streams(args) => streams::get(&client, args, printer),
        GetArgs::StreamComments(args) => streams::get_stream_comments(&client, args),
        GetArgs::Users(args) => users::get(&client, args, printer),
        GetArgs::CurrentUser => users::get_current_user(&client, printer),
        GetArgs::Quotas => quota::get(&client, printer),
    }
}
