use anyhow::{bail, Context, Result};

use chrono::{DateTime, Utc};
use colored::Colorize;
use reinfer_client::{
    resources::{bucket_statistics::Count, email::Email},
    BucketIdentifier, Client, EmailId, EmailsQueryFilter,
};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use crate::{
    printer::print_resources_as_json,
    progress::{Options as ProgressOptions, Progress},
};

#[derive(Debug, StructOpt)]
pub struct GetManyEmailsArgs {
    #[structopt(name = "bucket")]
    /// Bucket name or id
    bucket: BucketIdentifier,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write comments as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,

    #[structopt(name = "id")]
    /// Id of specific email to return
    id: Option<EmailId>,

    #[structopt(long = "mailbox")]
    /// Filter to emails belonging to this exact mailbox name.
    mailbox: Option<String>,

    #[structopt(long = "from-timestamp")]
    /// Include only emails with a timestamp greater than or equal to this
    /// (inclusive). RFC3339, e.g. 2024-01-01T00:00:00Z.
    from_timestamp: Option<DateTime<Utc>>,

    #[structopt(long = "to-timestamp")]
    /// Include only emails with a timestamp strictly less than this
    /// (exclusive). RFC3339, e.g. 2024-02-01T00:00:00Z.
    to_timestamp: Option<DateTime<Utc>>,
}

pub fn get_many(client: &Client, args: &GetManyEmailsArgs) -> Result<()> {
    let GetManyEmailsArgs {
        bucket,
        path,
        id,
        mailbox,
        from_timestamp,
        to_timestamp,
    } = args;

    let file = match path {
        Some(path) => Some(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        ),
        None => None,
    };

    if let Some(id) = id {
        if mailbox.is_some() || from_timestamp.is_some() || to_timestamp.is_some() {
            bail!("`--mailbox`, `--from-timestamp` and `--to-timestamp` cannot be combined with a specific email id.");
        }
        if let Some(file) = file {
            return download_email(client, bucket.clone(), id.clone(), file);
        } else {
            return download_email(client, bucket.clone(), id.clone(), io::stdout().lock());
        }
    }

    if let (Some(from), Some(to)) = (from_timestamp, to_timestamp) {
        if from > to {
            bail!("`--from-timestamp` must be less than or equal to `--to-timestamp`.");
        }
    }

    let filter = EmailsQueryFilter {
        from_timestamp: *from_timestamp,
        to_timestamp: *to_timestamp,
        mailbox_name: mailbox.clone(),
    };
    // `None` keeps the existing listing endpoint; any filter uses the query endpoint.
    let filter = (!filter.is_empty()).then_some(filter);

    if let Some(file) = file {
        download_emails(client, bucket.clone(), filter, file)
    } else {
        download_emails(client, bucket.clone(), filter, io::stdout().lock())
    }
}

fn download_email(
    client: &Client,
    bucket_identifier: BucketIdentifier,
    id: EmailId,
    mut writer: impl Write,
) -> Result<()> {
    let bucket = client
        .get_bucket(bucket_identifier)
        .context("Operation to get bucket has failed.")?;

    let response = client.get_email(&bucket.full_name(), id)?;

    print_resources_as_json(response, &mut writer)
}

fn download_emails(
    client: &Client,
    bucket_identifier: BucketIdentifier,
    filter: Option<EmailsQueryFilter>,
    mut writer: impl Write,
) -> Result<()> {
    let bucket = client
        .get_bucket(bucket_identifier)
        .context("Operation to get bucket has failed.")?;

    let statistics = Arc::new(Statistics::new());

    // An unfiltered listing knows the bucket's total up front and shows a
    // completion bar. A filtered query has no reliable total (under outline-mode
    // mailbox filtering a page can be short while more matches remain), so it
    // shows a count-only indicator.
    let total = match &filter {
        None => {
            let bucket_statistics = client
                .get_bucket_statistics(&bucket.full_name())
                .context("Could not get bucket statistics")?;
            Some(match bucket_statistics.count {
                Count::LowerBoundBucketCount { value } => value,
                Count::ExactBucketCount { value } => value,
            } as u64)
        }
        Some(_) => None,
    };

    let _progress = get_emails_progress_bar(total, &statistics);

    match filter {
        Some(filter) => drain_email_pages(
            client.query_emails_iter(&bucket.full_name(), filter, None),
            &statistics,
            &mut writer,
        )?,
        None => drain_email_pages(
            client.get_emails_iter(&bucket.full_name(), None),
            &statistics,
            &mut writer,
        )?,
    }

    log::info!(
        "Successfully downloaded {} emails.",
        statistics.num_downloaded(),
    );
    Ok(())
}

fn drain_email_pages(
    mut pages: impl Iterator<Item = reinfer_client::Result<Vec<Email>>>,
    statistics: &Statistics,
    mut writer: impl Write,
) -> Result<()> {
    pages.try_for_each(|page| {
        let page = page.context("Operation to get emails has failed.")?;
        statistics.add_emails(page.len());
        print_resources_as_json(page, &mut writer)
    })
}

#[derive(Debug)]
pub struct Statistics {
    downloaded: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            downloaded: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_emails(&self, num_downloaded: usize) {
        self.downloaded.fetch_add(num_downloaded, Ordering::SeqCst);
    }

    #[inline]
    fn num_downloaded(&self) -> usize {
        self.downloaded.load(Ordering::SeqCst)
    }
}

fn get_emails_progress_bar(total_bytes: Option<u64>, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistics| {
            let num_downloaded = statistics.num_downloaded();
            (
                num_downloaded as u64,
                format!(
                    "{} {}",
                    num_downloaded.to_string().bold(),
                    "emails".dimmed(),
                ),
            )
        },
        statistics,
        total_bytes,
        ProgressOptions { bytes_units: false },
    )
}
