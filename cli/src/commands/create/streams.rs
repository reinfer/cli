use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    resources::stream::{NewStream, UserModelVersion},
    Client, DatasetIdentifier,
};

use structopt::StructOpt;

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
    model_version: UserModelVersion,
}

pub fn create(client: &Client, args: &CreateStreamsArgs) -> Result<()> {
    let CreateStreamsArgs {
        path,
        dataset_id,
        model_version,
    } = args;

    let file = BufReader::new(
        File::open(path).with_context(|| format!("Could not open file `{}`", path.display()))?,
    );

    let dataset = client.get_dataset(dataset_id.clone())?;

    for read_stream_result in read_streams_iter(file) {
        let mut new_stream = read_stream_result?;

        new_stream.set_model_version(model_version);

        client.put_stream(&dataset.full_name(), &new_stream)?;
        info!("Created stream {}", new_stream.name.0)
    }
    Ok(())
}

fn read_streams_iter<'a>(
    mut streams: impl BufRead + 'a,
) -> impl Iterator<Item = Result<NewStream>> + 'a {
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
            serde_json::from_str::<NewStream>(line.trim_end()).with_context(|| {
                format!("Could not parse stream at line {line_number} from input stream")
            }),
        )
    })
}
