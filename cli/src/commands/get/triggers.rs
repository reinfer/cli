use anyhow::{Context, Result};
use reinfer_client::{Client, DatasetIdentifier, TriggerFullName};
use std::io;
use structopt::StructOpt;

use crate::printer::{print_resources_as_json, Printer};

#[derive(Debug, StructOpt)]
pub struct GetTriggersArgs {
    #[structopt(short = "d", long = "dataset")]
    /// The dataset name or id
    dataset: DatasetIdentifier,
}

#[derive(Debug, StructOpt)]
pub struct GetTriggerCommentsArgs {
    #[structopt(short = "t", long = "trigger")]
    /// The full trigger name `<owner>/<dataset>/<trigger>`.
    trigger: TriggerFullName,

    #[structopt(short = "s", long = "size", default_value = "16")]
    /// The max number of comments to return per batch.
    size: u32,

    #[structopt(short = "l", long = "listen")]
    /// If set, the command will run forever polling every N seconds and advancing the trigger.
    listen: Option<f64>,

    #[structopt(long = "individual-advance")]
    /// If set, the command will acknowledge each comment in turn, rather than full batches.
    individual_advance: bool,
}

pub async fn get(client: &Client, args: &GetTriggersArgs, printer: &Printer) -> Result<()> {
    let GetTriggersArgs { dataset } = args;
    let dataset_name = client
        .get_dataset(dataset.clone())
        .await
        .context("Operation to get dataset has failed.")?
        .full_name();
    let mut triggers = client
        .get_triggers(&dataset_name)
        .await
        .context("Operation to list triggers has failed.")?;
    triggers.sort_unstable_by(|lhs, rhs| lhs.name.0.cmp(&rhs.name.0));
    printer.print_resources(&triggers)
}

pub async fn get_trigger_comments(client: &Client, args: &GetTriggerCommentsArgs) -> Result<()> {
    let GetTriggerCommentsArgs {
        trigger,
        size,
        listen,
        individual_advance,
    } = args;

    match listen {
        Some(delay) => loop {
            let batch = client
                .fetch_trigger_comments(trigger, *size)
                .await
                .context("Operation to fetch trigger comments failed.")?;
            if batch.results.is_empty() {
                if batch.filtered == 0 {
                    std::thread::sleep(std::time::Duration::from_secs_f64(*delay));
                } else {
                    client
                        .advance_trigger(trigger, batch.sequence_id)
                        .await
                        .context("Operation to advance trigger for batch failed.")?;
                }
                continue;
            }
            let needs_final_advance = !individual_advance
                || batch.sequence_id != batch.results.last().unwrap().sequence_id;
            for result in batch.results {
                print_resources_as_json(Some(&result), io::stdout().lock())?;

                if *individual_advance {
                    client
                        .advance_trigger(trigger, result.sequence_id)
                        .await
                        .context("Operation to advance trigger for comment failed.")?;
                }
            }
            if needs_final_advance {
                client
                    .advance_trigger(trigger, batch.sequence_id)
                    .await
                    .context("Operation to advance trigger for batch failed.")?;
            }
        },
        None => {
            let batch = client
                .fetch_trigger_comments(trigger, *size)
                .await
                .context("Operation to fetch trigger comments failed.")?;
            print_resources_as_json(Some(&batch), io::stdout().lock())
        }
    }
}
