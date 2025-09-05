use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{Context, Result};
use log::info;
use openapi::{
    apis::{
        configuration::Configuration,
        datasets_api::get_dataset,
        streams_api::create_stream,
    },
    models::{CreateStreamRequest, TriggerNew, TriggerUserModel},
};
use structopt::StructOpt;

use crate::utils::resource_identifier::DatasetIdentifier;

#[derive(Debug, StructOpt)]
pub struct CreateStreamsArgs {
    #[structopt(short = "d", long = "dataset")]
    /// Dataset where the streams should be created
    dataset_id: DatasetIdentifier,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with streams
    path: PathBuf,

    #[structopt(short = "v", long = "model-version")]
    /// The model version for the new streams to use
    model_version: TriggerUserModel,
}

pub fn create(config: &Configuration, args: &CreateStreamsArgs) -> Result<()> {
    let CreateStreamsArgs {
        path,
        dataset_id,
        model_version,
    } = args;

    let file = BufReader::new(
        File::open(path).with_context(|| format!("Could not open file `{}`", path.display()))?,
    );

    // Get dataset info
    let dataset = match &dataset_id {
        DatasetIdentifier::Id(_) => {
            return Err(anyhow::anyhow!("Dataset lookup by ID is not supported. Please use the full name format 'owner/dataset'"));
        }
        DatasetIdentifier::FullName(full_name) => {
            get_dataset(config, full_name.owner(), full_name.name())
                .context("Failed to get dataset")?
                .dataset
        }
    };

    for read_stream_result in read_streams_iter(file) {
        let mut trigger_new = read_stream_result?;

        // Set the model version
        trigger_new.model = Some(Some(Box::new(*model_version)));

        let request = CreateStreamRequest {
            stream: Box::new(trigger_new.clone()),
        };

        create_stream(config, &dataset.owner, &dataset.name, request)
            .context("Failed to create stream")?;
        info!("Created stream {}", trigger_new.name.as_deref().unwrap_or("unnamed"))
    }
    Ok(())
}

fn read_streams_iter<'a>(
    mut streams: impl BufRead + 'a,
) -> impl Iterator<Item = Result<TriggerNew>> + 'a {
    let mut line = String::new();
    let mut line_number: u32 = 0;
    std::iter::from_fn(move || {
        line_number += 1;
        line.clear();

        let read_result = streams
            .read_line(&mut line)
            .with_context(|| format!("Could not read line {line_number} from input stream"));

        match read_result {
            Ok(0) => return None,
            Err(e) => return Some(Err(e)),
            _ => {}
        }

        Some(
            serde_json::from_str::<TriggerNew>(line.trim_end()).with_context(|| {
                format!("Could not parse stream at line {line_number} from input stream")
            }),
        )
    })
}
