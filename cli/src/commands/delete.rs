use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketIdentifier, Client, CommentId, DatasetIdentifier, SourceIdentifier};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum DeleteArgs {
    #[structopt(name = "source")]
    /// Delete a source
    Source {
        #[structopt(name = "source")]
        /// Name or id of the source to delete
        source: SourceIdentifier,
    },

    #[structopt(name = "comments")]
    /// Delete comments by id in a source.
    Comments {
        #[structopt(short = "s", long = "source")]
        /// Name or id of the source to delete comments from
        source: SourceIdentifier,

        #[structopt(name = "comment id")]
        /// Ids of the comments to delete
        comments: Vec<CommentId>,
    },

    #[structopt(name = "bucket")]
    /// Delete a bucket
    Bucket {
        #[structopt(name = "bucket")]
        /// Name or id of the bucket to delete
        bucket: BucketIdentifier,
    },

    #[structopt(name = "dataset")]
    /// Delete a dataset
    Dataset {
        #[structopt(name = "dataset")]
        /// Name or id of the dataset to delete
        dataset: DatasetIdentifier,
    },
}

pub fn run(delete_args: &DeleteArgs, client: Client) -> Result<()> {
    match delete_args {
        DeleteArgs::Source { source } => {
            client
                .delete_source(source.clone())
                .context("Operation to delete source has failed.")?;
            info!("Deleted source.");
        }
        DeleteArgs::Comments { source, comments } => {
            client
                .delete_comments(source.clone(), comments)
                .context("Operation to delete comments has failed.")?;
            info!("Deleted comments.");
        }
        DeleteArgs::Dataset { dataset } => {
            client
                .delete_dataset(dataset.clone())
                .context("Operation to delete dataset has failed.")?;
            info!("Deleted dataset.");
        }
        DeleteArgs::Bucket { bucket } => {
            client
                .delete_bucket(bucket.clone())
                .context("Operation to delete bucket has failed.")?;
            info!("Deleted bucket.");
        }
    };
    Ok(())
}
