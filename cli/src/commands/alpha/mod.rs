mod take_more_labels;

use reinfer_client::Client;
use structopt::StructOpt;

use self::take_more_labels::TakeMoreLabelsArgs;
use crate::errors::Result;

#[derive(Debug, StructOpt)]
pub enum AlphaArgs {
    #[structopt(name = "take-more-labels")]
    /// Oneoff task. Take labels from a source dataset and merge them to a target dataset,
    /// only for target comments with a certain label assigned.
    TakeMoreLabels(TakeMoreLabelsArgs),
}

pub fn run(alpha_args: &AlphaArgs, client: Client) -> Result<()> {
    match alpha_args {
        AlphaArgs::TakeMoreLabels(take_more_labels_args) => {
            take_more_labels::run(take_more_labels_args, &client)
        }
    }
}
