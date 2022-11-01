use anyhow::{Context, Result};
use colored::Colorize;
use log::info;
use reinfer_client::{Client, UpdateUser, UserId};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use crate::progress::{Options as ProgressOptions, Progress};

#[derive(Debug, StructOpt)]
pub struct UpdateUsersArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with users. If not specified, stdin will be used.
    input_file: Option<PathBuf>,

    #[structopt(long)]
    /// Don't display a progress bar (only applicable when --file is used).
    no_progress: bool,
}

pub fn update(client: &Client, args: &UpdateUsersArgs) -> Result<()> {
    let statistics = match &args.input_file {
        Some(input_file) => {
            info!("Processing users from file `{}`", input_file.display(),);
            let file_metadata = fs::metadata(&input_file).with_context(|| {
                format!("Could not get file metadata for `{}`", input_file.display())
            })?;
            let file = BufReader::new(
                File::open(input_file)
                    .with_context(|| format!("Could not open file `{}`", input_file.display()))?,
            );
            let statistics = Arc::new(Statistics::new());
            let progress = if args.no_progress {
                None
            } else {
                Some(progress_bar(file_metadata.len(), &statistics))
            };
            update_users_from_reader(client, file, &statistics)?;
            if let Some(mut progress) = progress {
                progress.done();
            }
            Arc::try_unwrap(statistics).unwrap()
        }
        None => {
            info!("Processing users from stdin",);
            let statistics = Statistics::new();
            update_users_from_reader(client, BufReader::new(io::stdin()), &statistics)?;
            statistics
        }
    };

    info!(
        concat!("Successfully updated {} users",),
        statistics.num_updated(),
    );

    Ok(())
}

use serde::{self, Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
struct UserLine {
    id: UserId,

    #[serde(flatten)]
    update: UpdateUser,
}

fn update_users_from_reader(
    client: &Client,
    mut users: impl BufRead,
    statistics: &Statistics,
) -> Result<()> {
    let mut line_number = 1;
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = users
            .read_line(&mut line)
            .with_context(|| format!("Could not read line {} from input stream", line_number))?;

        if bytes_read == 0 {
            return Ok(());
        }

        statistics.add_bytes_read(bytes_read);
        let user_line = serde_json::from_str::<UserLine>(line.trim_end()).with_context(|| {
            format!(
                "Could not parse user at line {} from input stream",
                line_number,
            )
        })?;

        // Upload users
        client
            .post_user(&user_line.id, user_line.update)
            .context("Could not update user")?;
        statistics.add_user();

        line_number += 1;
    }
}

#[derive(Debug)]
pub struct Statistics {
    bytes_read: AtomicUsize,
    updated: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            bytes_read: AtomicUsize::new(0),
            updated: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_bytes_read(&self, bytes_read: usize) {
        self.bytes_read.fetch_add(bytes_read, Ordering::SeqCst);
    }

    #[inline]
    fn add_user(&self) {
        self.updated.fetch_add(1, Ordering::SeqCst);
    }

    #[inline]
    fn bytes_read(&self) -> usize {
        self.bytes_read.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_updated(&self) -> usize {
        self.updated.load(Ordering::SeqCst)
    }
}

fn progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let bytes_read = statistics.bytes_read();
            let num_updated = statistics.num_updated();
            (
                bytes_read as u64,
                format!("{} {}", num_updated.to_string().bold(), "users".dimmed()),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: true },
    )
}
