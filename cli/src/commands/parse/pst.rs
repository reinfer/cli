use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use colored::Colorize;
use itertools::Itertools;
use mailparse::{parse_header, MailHeader};
use reinfer_client::{
    resources::{attachments::AttachmentMetadata, email::EmailMetadata},
    BucketIdentifier, Client, EmailId, Mailbox, MimeContent, NewEmail,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use crate::{
    commands::ensure_uip_user_consents_to_ai_unit_charge,
    parse::pff::{LibPffAttachmentType, PstFile},
    progress::{Options as ProgressOptions, Progress},
};

use super::pff::PstMessage;

#[derive(Debug, StructOpt)]
pub struct ParsePstArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to the pst
    pst_path: PathBuf,

    #[structopt(long = "batch-size", default_value = "128")]
    /// Number of emails to batch in a single request.
    batch_size: usize,

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,

    #[structopt(short = "b", long = "bucket")]
    /// Name of the bucket where the emails will be uploaded.
    bucket: BucketIdentifier,

    #[structopt(long = "resume-on-error")]
    /// Whether to attempt to resume processing on error
    resume_on_error: bool,

    #[structopt(short = "y", long = "yes")]
    /// Consent to ai unit charge. Suppresses confirmation prompt.
    yes: bool,

    #[structopt(short = "d", long = "dry-run")]
    /// Print any parsing errors without uploading the pst
    dry_run: bool,
}

#[derive(Debug)]
pub struct Statistics {
    uploaded: AtomicUsize,
    failed_to_parse: AtomicUsize,
    failed_to_upload: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            uploaded: AtomicUsize::new(0),
            failed_to_parse: AtomicUsize::new(0),
            failed_to_upload: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_failed_to_parse(&self, num_failed: usize) {
        self.failed_to_parse.fetch_add(num_failed, Ordering::SeqCst);
    }

    #[inline]
    fn add_failed_to_upload(&self, num_failed: usize) {
        self.failed_to_upload
            .fetch_add(num_failed, Ordering::SeqCst);
    }

    #[inline]
    fn add_uploaded(&self, num_uploaded: usize) {
        self.uploaded.fetch_add(num_uploaded, Ordering::SeqCst);
    }

    #[inline]
    fn num_uploaded(&self) -> usize {
        self.uploaded.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_to_parse(&self) -> usize {
        self.failed_to_parse.load(Ordering::SeqCst)
    }

    #[inline]
    fn num_failed_to_upload(&self) -> usize {
        self.failed_to_upload.load(Ordering::SeqCst)
    }
}

fn get_progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistic| {
            let num_uploaded = statistic.num_uploaded();
            let num_failed_to_parse = statistic.num_failed_to_parse();
            let num_failed_to_upload = statistic.num_failed_to_upload();

            let failed_to_parse_part = if num_failed_to_parse > 0 {
                format!(
                    " {} {}",
                    num_failed_to_parse.to_string().bold(),
                    "failed to parse".dimmed()
                )
            } else {
                String::new()
            };

            let failed_to_upload_part = if num_failed_to_upload > 0 {
                format!(
                    " {} {}",
                    num_failed_to_upload.to_string().bold(),
                    "failed to upload".dimmed()
                )
            } else {
                String::new()
            };

            (
                num_uploaded as u64,
                format!(
                    "{} {} {} {}",
                    num_uploaded.to_string().bold(),
                    "processed".dimmed(),
                    failed_to_parse_part,
                    failed_to_upload_part
                ),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: false },
    )
}

pub fn parse(client: &Client, args: &ParsePstArgs) -> Result<()> {
    let statistics = Arc::new(Statistics::new());

    let mut errors = HashMap::<String, usize>::new();

    if !args.no_charge && !args.yes {
        ensure_uip_user_consents_to_ai_unit_charge(client.base_url())?;
    }

    log::info!("Opening pst file...");
    let pst = PstFile::open(&args.pst_path).context("Could not open PST file")?;
    let root_folder = pst
        .get_root_folder()
        .context("Could not get PST root folder")?;

    log::info!("Counting total items...");
    let item_count = root_folder.clone().get_item_count()?;

    let items_iter = root_folder
        .all_items_iter()
        .context("Could not get root folder iter")?
        .chunks(args.batch_size);

    let _progress = get_progress_bar(item_count as u64, &statistics);

    let pst_file_name = args
        .pst_path
        .file_name()
        .context("Could not get pst file name")?
        .to_string_lossy()
        .to_string();

    log::info!("Starting processing...");
    for batch in &items_iter {
        let mut emails = Vec::new();
        for pst_message in batch {
            match pst_message {
                Ok(message) => {
                    match pst_message_to_new_email(message, Mailbox(pst_file_name.clone())) {
                        Ok(email) => emails.push(email),
                        Err(e) => {
                            if !args.resume_on_error && !args.dry_run {
                                return Err(e);
                            } else {
                                statistics.add_failed_to_parse(1);
                                *errors.entry(e.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    if !args.resume_on_error && !args.dry_run {
                        return Err(e);
                    } else {
                        statistics.add_failed_to_parse(1);
                    }
                }
            }
        }

        let batch_len = emails.len();
        if !args.dry_run {
            let bucket = client.get_bucket(args.bucket.clone())?;
            if args.resume_on_error {
                let result =
                    client.put_emails_split_on_failure(&bucket.full_name(), emails, args.no_charge);
                statistics.add_uploaded(batch_len - result.num_failed);
                statistics.add_failed_to_upload(result.num_failed);
            } else {
                client.put_emails(&bucket.full_name(), emails, args.no_charge)?;
                statistics.add_uploaded(batch_len);
            };
        } else {
            statistics.add_uploaded(batch_len);
        }
    }
    if args.dry_run {
        if !errors.is_empty() {
            let errors_msg = errors
                .iter()
                .map(|(error, count)| format!("{count} failed to parse due to the error: {error}"))
                .join("\n");

            log::error!("Parse errors found:\n\n{errors_msg}");
        } else {
            log::info!("No parse errors found");
        }
    } else {
        if statistics.num_failed_to_parse() > 0 {
            log::warn!(
                "{} emails failed to parse.",
                statistics.num_failed_to_parse()
            )
        }

        if statistics.num_failed_to_upload() > 0 {
            log::warn!(
                "{} emails failed to upload.",
                statistics.num_failed_to_upload()
            )
        }

        log::info!(
            "{} emails uploaded successfully.",
            statistics.num_uploaded()
        );
    }

    Ok(())
}

pub fn pst_message_to_new_email(pst_message: PstMessage, mailbox: Mailbox) -> Result<NewEmail> {
    // Parse Headers
    let raw_headers = pst_message
        .get_transport_headers()?
        .context("Could not read transport headers. Sent items are dropped when psts are exported from outlook. Please export from exchange.")?;

    let (parsed_headers, _) = mailparse::parse_headers(raw_headers.as_bytes())?;

    // Get Message ID
    let message_id = PstMessage::expect_header(&parsed_headers, "Message-ID")?;
    let id = EmailId(message_id);

    // Get timestamp
    pub fn parse_date_header_string(date_str: String) -> Result<DateTime<Utc>> {
        pub fn truncate_string(string: &str, max_chars: usize) -> &str {
            // Worst case all chars take 4 bytes as UTF-8.
            if string.len() < max_chars / 4 {
                return string;
            }
            match string.char_indices().nth(max_chars) {
                None => string,
                Some((index, _)) => &string[..index],
            }
        }

        // Some dates contain the timezone as a string at the end (not strictly
        // rfc2822 or at least `parse_from_rfc2822` doesn't like it).
        // Example: Mon, 15 Apr 2019 14:05:23 +0000 (UTC)
        //
        // Truncating at 31 may leave one empty space at the end if the date only
        // has 1 digit, so we have call `trim()` before passing it to
        // `parse_from_rfc2822`.
        let date_str = truncate_string(&date_str, 31);

        // According to the rfc, the correct interpretation of -0000 is that the
        // timezone is unknown. The code below instead interprets this as UTC.
        // For further information, see https://github.com/chronotope/chrono/issues/102
        let date_str = date_str.replace("-0000", "+0000");
        if let Ok(date) = DateTime::parse_from_rfc2822(date_str.trim()) {
            return Ok(date.with_timezone(&Utc));
        }
        // chrono's email date parsing often fails, so use another method in that case:
        if let Ok(epoch) = mailparse::dateparse(date_str.trim()) {
            return if epoch == 0 {
                // `mailparse` will return zero epoch in all sorts of error scenarios, like when parsing
                // "asdf".
                Err(anyhow!("Date header value {} is invalid", date_str))
            } else if let Some(datetime) = Utc.timestamp_opt(epoch, 0).single() {
                Ok(datetime)
            } else {
                Err(anyhow!("Date header value {} is invalid", date_str))
            };
        }
        Err(anyhow!("Date header value {} is invalid", date_str))
    }

    let date_str = PstMessage::expect_header(&parsed_headers, "Date")?;
    let timestamp = parse_date_header_string(date_str)?;

    // Get Attachments
    let mut attachments = Vec::new();

    for attachment in pst_message.attachments_iter()? {
        let attachment = attachment?;

        if attachment.attachment_type != LibPffAttachmentType::Data {
            continue;
        }

        attachments.push(AttachmentMetadata {
            name: attachment.get_name()?,
            content_type: attachment.get_content_type()?,
            size: attachment.get_size()? as u64,
            attachment_reference: None,
            content_hash: None,
        });
    }

    // Get Metadata
    let metadata = Some(EmailMetadata {
        folder: Some(pst_message.folder.0.clone()),
        has_attachments: Some(!attachments.is_empty()),
        ..Default::default()
    });

    // Get Mime content
    // Replace content type header with html utf8
    let parsed_headers: Vec<MailHeader<'_>> = parsed_headers
        .into_iter()
        .map(|header| {
            if header.get_key() == "Content-Type" {
                let (content_type_header, _) =
                    parse_header(b"Content-Type: text/html; charset=UTF-8")
                        .expect("Could not parse default content type header");
                content_type_header
            } else {
                header
            }
        })
        .collect();

    // Get Headers
    let headers_as_mime_string = parsed_headers
        .iter()
        .map(|header| format!("{}: {}", header.get_key(), header.get_value()))
        .collect::<Vec<String>>()
        .join("\r\n");

    // Get body
    let body = if let Some(html_body) = pst_message.get_html_body()? {
        html_body
    } else {
        pst_message
            .get_plain_text_body()?
            .context("Plain text and html body missing for message. Rtf content not supported.")?
    };

    let mime_content = MimeContent(format!("{headers_as_mime_string}\r\n\r\n{body}"));

    Ok(NewEmail {
        id,
        attachments,
        mailbox,
        metadata,
        timestamp,
        mime_content,
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use reinfer_client::NewEmail;
    use std::path::Path;

    #[test]
    fn test_parse_pst() {
        let pst_file =
            PstFile::open(Path::new("tests/samples/test.pst")).expect("Could not open pst file");

        let root_folder = pst_file
            .get_root_folder()
            .expect("Could not get test pst root folder");

        let emails = root_folder
            .all_items_iter()
            .expect("Could not get all items iter")
            .map(|message| {
                pst_message_to_new_email(
                    message.expect("Could not parse message"),
                    Mailbox("test.pst".to_string()),
                )
                .expect("Could not create new email from pst message")
            })
            .collect::<Vec<NewEmail>>();

        let expected_emails: Vec<NewEmail> =
            serde_json::from_str(include_str!("../../../tests/samples/test.pst.json"))
                .expect("Could not read expected emails");

        assert_eq!(expected_emails, emails);
    }
}
