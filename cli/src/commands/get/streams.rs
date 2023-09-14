use anyhow::{Context, Result};
use reinfer_client::{Client, DatasetIdentifier, StreamFullName};
use std::{
    fs::File,
    io,
    io::{BufWriter, Write},
    path::PathBuf,
};
use structopt::StructOpt;

use crate::printer::{print_resources_as_json, Printer};

#[derive(Debug, StructOpt)]
pub struct GetStreamsArgs {
    #[structopt(short = "d", long = "dataset")]
    /// The dataset name or id
    dataset: DatasetIdentifier,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write streams as JSON.
    path: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct GetStreamCommentsArgs {
    #[structopt(long = "stream")]
    /// The full stream name `<owner>/<dataset>/<stream>`.
    stream: StreamFullName,

    #[structopt(long = "size", default_value = "16")]
    /// The max number of comments to return per batch.
    size: u32,

    #[structopt(long = "listen")]
    /// If set, the command will run forever polling every N seconds and advancing the stream.
    listen: Option<f64>,

    #[structopt(long = "individual-advance")]
    /// If set, the command will acknowledge each comment in turn, rather than full batches.
    individual_advance: bool,
}

pub fn get(client: &Client, args: &GetStreamsArgs, printer: &Printer) -> Result<()> {
    let GetStreamsArgs { dataset, path } = args;

    let file: Option<Box<dyn Write>> = match path {
        Some(path) => Some(Box::new(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        )),
        None => None,
    };

    let dataset_name = client
        .get_dataset(dataset.clone())
        .context("Operation to get dataset has failed.")?
        .full_name();
    let mut streams = client
        .get_streams(&dataset_name)
        .context("Operation to list streams has failed.")?;
    streams.sort_unstable_by(|lhs, rhs| lhs.name.0.cmp(&rhs.name.0));

    if let Some(file) = file {
        print_resources_as_json(streams, file)
    } else {
        printer.print_resources(&streams)
    }
}

pub fn get_stream_comments(client: &Client, args: &GetStreamCommentsArgs) -> Result<()> {
    let GetStreamCommentsArgs {
        stream,
        size,
        listen,
        individual_advance,
    } = args;

    match listen {
        Some(delay) => loop {
            let batch = client
                .fetch_stream_comments(stream, *size)
                .context("Operation to fetch stream comments failed.")?;
            if batch.results.is_empty() {
                if batch.filtered == 0 {
                    std::thread::sleep(std::time::Duration::from_secs_f64(*delay));
                } else {
                    client
                        .advance_stream(stream, batch.sequence_id)
                        .context("Operation to advance stream for batch failed.")?;
                }
                continue;
            }
            let needs_final_advance = !individual_advance
                || batch.sequence_id != batch.results.last().unwrap().sequence_id;
            for result in batch.results {
                print_resources_as_json(Some(&result), io::stdout().lock())?;

                if *individual_advance {
                    client
                        .advance_stream(stream, result.sequence_id)
                        .context("Operation to advance stream for comment failed.")?;
                }
            }
            if needs_final_advance {
                client
                    .advance_stream(stream, batch.sequence_id)
                    .context("Operation to advance stream for batch failed.")?;
            }
        },
        None => {
            let batch = client
                .fetch_stream_comments(stream, *size)
                .context("Operation to fetch stream comments failed.")?;
            print_resources_as_json(Some(&batch), io::stdout().lock())
        }
    }
}
