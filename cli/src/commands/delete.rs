use failchain::ResultExt;
use log::info;
use reinfer_client::{BucketIdentifier, Client, DatasetIdentifier, SourceIdentifier};
use structopt::StructOpt;

use crate::errors::{ErrorKind, Result};

#[derive(Debug, StructOpt)]
pub enum DeleteArgs {
    #[structopt(name = "source")]
    /// Delete a source
    Source {
        #[structopt(name = "source")]
        /// Name or id of the source to delete
        source: SourceIdentifier,
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
            client.delete_source(source.clone()).chain_err(|| {
                ErrorKind::Unknown("Operation to delete source has failed.".into())
            })?;
            info!("Deleted source.");
        }
        DeleteArgs::Dataset { dataset } => {
            client.delete_dataset(dataset.clone()).chain_err(|| {
                ErrorKind::Unknown("Operation to delete dataset has failed.".into())
            })?;
            info!("Deleted dataset.");
        }
        DeleteArgs::Bucket { bucket } => {
            client.delete_bucket(bucket.clone()).chain_err(|| {
                ErrorKind::Unknown("Operation to delete bucket has failed.".into())
            })?;
            info!("Deleted bucket.");
        }
    };
    Ok(())
}
