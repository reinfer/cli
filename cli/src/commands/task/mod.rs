mod take_more_labels;

use reinfer_client::Client;
use structopt::StructOpt;

use self::take_more_labels::TakeMoreLabelsArgs;
use crate::errors::Result;

#[derive(Debug, StructOpt)]
pub enum TaskArgs {
    #[structopt(name = "take-more-labels")]
    /// Take labels from a source dataset and merge them to a target dataset, only for
    /// target comments with a certain label assigned.
    TakeMoreLabels(TakeMoreLabelsArgs),
}

pub fn run(task_args: &TaskArgs, client: Client, experimental: bool) -> Result<()> {
    match task_args {
        TaskArgs::TakeMoreLabels(take_more_labels_args) => {
            take_more_labels::run(take_more_labels_args, &client, experimental)
        }
    }
}
