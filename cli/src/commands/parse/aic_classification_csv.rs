use crate::{
    commands::{
        create::annotations::{upload_batch_of_annotations, CommentIdComment, NewAnnotation},
        parse::{get_progress_bar, upload_batch_of_comments},
    },
    parse::Statistics,
};
use anyhow::Result;
use log::{error, info};
use scoped_threadpool::Pool;
use serde::Deserialize;
use std::sync::{mpsc::channel, Arc};

use reinfer_client::{
    Client, CommentId, DatasetFullName, DatasetIdentifier, EitherLabelling, Label, Message,
    MessageBody, NewComment, NewLabelling, Source, SourceIdentifier, DEFAULT_LABEL_GROUP_NAME,
};
use std::path::PathBuf;
use structopt::StructOpt;

const UPLOAD_BATCH_SIZE: usize = 4;

#[derive(Debug, StructOpt)]
pub struct ParseAicClassificationCsvArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to the csv to parse
    file_path: PathBuf,

    #[structopt(short = "s", long = "source")]
    /// The source to upload the data to
    source: SourceIdentifier,

    #[structopt(short = "d", long = "dataset")]
    /// The dataset to upload annotations to
    dataset: DatasetIdentifier,

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,
}

#[derive(Deserialize)]
pub struct AicClassificationRecord {
    input: String,
    target: String,
}

#[allow(clippy::too_many_arguments)]
fn send_comments_if_needed(
    comments: &mut Vec<NewComment>,
    annotations: &mut Vec<NewAnnotation>,
    force_send: bool,
    pool: &mut Pool,
    client: &Client,
    source: &Source,
    statistics: &Statistics,
    dataset: &DatasetFullName,
    no_charge: bool,
) -> Result<()> {
    let thread_count = pool.thread_count();
    let should_upload = comments.len() > (thread_count as usize * UPLOAD_BATCH_SIZE);

    if !force_send && !should_upload {
        return Ok(());
    }

    let chunks: Vec<_> = comments.chunks(UPLOAD_BATCH_SIZE).collect();

    let (error_sender, error_receiver) = channel();
    pool.scoped(|scope| {
        for chunk in chunks {
            scope.execute(|| {
                let result = upload_batch_of_comments(client, source, chunk, no_charge, statistics);

                if let Err(error) = result {
                    error_sender.send(error).expect("Could not send error");
                }
            });
        }
    });

    upload_batch_of_annotations(
        annotations,
        client,
        source,
        statistics,
        dataset,
        pool,
        false,
    )?;

    if let Ok(error) = error_receiver.try_recv() {
        Err(error)
    } else {
        comments.clear();
        annotations.clear();
        Ok(())
    }
}

pub fn parse(client: &Client, args: &ParseAicClassificationCsvArgs, pool: &mut Pool) -> Result<()> {
    let ParseAicClassificationCsvArgs {
        file_path,
        source,
        dataset,
        no_charge,
    } = args;

    let source = client.get_source(source.clone())?;
    let dataset = client.get_dataset(dataset.clone())?;
    let record_count = csv::Reader::from_path(file_path)?.records().count();

    let statistics = Arc::new(Statistics::new());
    let _progress = get_progress_bar(record_count as u64, &statistics);

    let mut reader = csv::Reader::from_path(file_path)?;

    let headers = reader.headers()?.clone();

    let mut comments: Vec<NewComment> = Vec::new();
    let mut annotations: Vec<NewAnnotation> = Vec::new();
    for (idx, row) in reader.records().enumerate() {
        match row {
            Ok(row) => {
                let record: AicClassificationRecord = row.deserialize(Some(&headers))?;
                let comment_id = CommentId(idx.to_string());

                comments.push(NewComment {
                    id: comment_id.clone(),
                    timestamp: chrono::Utc::now(),
                    messages: vec![Message {
                        body: MessageBody {
                            text: record.input,
                            ..Default::default()
                        },
                        ..Default::default()
                    }],
                    ..Default::default()
                });
                annotations.push(NewAnnotation {
                    comment: CommentIdComment { id: comment_id },
                    labelling: Some(EitherLabelling::Labelling(vec![NewLabelling {
                        group: DEFAULT_LABEL_GROUP_NAME.clone(),
                        assigned: Some(vec![Label {
                            name: reinfer_client::LabelName(record.target),
                            sentiment: reinfer_client::Sentiment::Positive,
                            metadata: None,
                        }]),
                        dismissed: None,
                    }])),
                    entities: None,
                    moon_forms: None,
                });

                send_comments_if_needed(
                    &mut comments,
                    &mut annotations,
                    false,
                    pool,
                    client,
                    &source,
                    &statistics,
                    &dataset.full_name(),
                    *no_charge,
                )?;
                statistics.increment_processed()
            }
            Err(_) => {
                error!("Failed to process row {idx}");
                statistics.increment_failed();
                statistics.increment_processed()
            }
        }
    }
    send_comments_if_needed(
        &mut comments,
        &mut annotations,
        true,
        pool,
        client,
        &source,
        &statistics,
        &dataset.full_name(),
        *no_charge,
    )?;

    info!(
        "Uploaded {}. {} Failed",
        statistics.num_uploaded(),
        statistics.num_failed()
    );
    Ok(())
}
