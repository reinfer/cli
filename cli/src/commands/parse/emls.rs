use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use log::error;
use mailparse::{DispositionType, MailHeader, MailHeaderMap};
use scoped_threadpool::Pool;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{mpsc::channel, Arc},
};

use crate::commands::{
    ensure_uip_user_consents_to_ai_unit_charge,
    parse::{get_files_in_directory, get_progress_bar, upload_batch_of_new_emails, Statistics},
};
use crate::utils::{types::identifiers::BucketIdentifier, types::names::FullName};
use openapi::{
    apis::{buckets_api::get_bucket, configuration::Configuration},
    models::{Attachment, EmailNew},
};
use structopt::StructOpt;

const UPLOAD_BATCH_SIZE: usize = 4;

#[derive(Debug, StructOpt)]
pub struct ParseEmlArgs {
    #[structopt(short = "d", long = "dir", parse(from_os_str))]
    /// Directory containing the emls
    directory: PathBuf,

    #[structopt(short = "b", long = "bucket")]
    /// Name of the bucket where the emails will be uploaded.
    bucket: BucketIdentifier,

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,

    #[structopt(short = "y", long = "yes")]
    /// Consent to ai unit charge. Suppresses confirmation prompt.
    yes: bool,
}

pub fn parse(config: &Configuration, args: &ParseEmlArgs, pool: &mut Pool) -> Result<()> {
    let ParseEmlArgs {
        directory,
        bucket,
        no_charge,
        yes,
    } = args;

    if !no_charge && !yes {
        ensure_uip_user_consents_to_ai_unit_charge(&config.base_path.parse()?)?;
    }

    let eml_paths = get_files_in_directory(directory, "eml", true)?;
    let statistics = Arc::new(Statistics::new());
    let _progress = get_progress_bar(eml_paths.len() as u64, &statistics);

    let bucket_response = get_bucket(config, bucket.owner().unwrap(), bucket.name().unwrap())
        .with_context(|| format!("Unable to get bucket {}", args.bucket))?;
    let bucket = bucket_response.bucket;

    let mut emails = Vec::new();
    let mut errors = Vec::new();

    let mut send_if_needed = |emails: &mut Vec<EmailNew>, force_send: bool| -> Result<()> {
        let thread_count = pool.thread_count();
        let should_upload = emails.len() > (thread_count as usize * UPLOAD_BATCH_SIZE);

        if !force_send && !should_upload {
            return Ok(());
        }

        let chunks: Vec<_> = emails.chunks(UPLOAD_BATCH_SIZE).collect();

        let (error_sender, error_receiver) = channel();
        pool.scoped(|scope| {
            for chunk in chunks {
                scope.execute(|| {
                    // Create BucketIdentifier from existing bucket info
                    let bucket_id = BucketIdentifier::FullName(FullName(format!(
                        "{}/{}",
                        bucket.owner, bucket.name
                    )));

                    let result = upload_batch_of_new_emails(
                        config,
                        &bucket_id,
                        chunk,
                        *no_charge,
                        &statistics,
                    );

                    if let Err(error) = result {
                        error_sender.send(error).expect("Could not send error");
                    }
                });
            }
        });

        if let Ok(error) = error_receiver.try_recv() {
            Err(error)
        } else {
            emails.clear();
            Ok(())
        }
    };

    for path in eml_paths {
        match read_eml_to_new_email(&path.path()) {
            Ok(new_email) => {
                emails.push(new_email);

                send_if_needed(&mut emails, false)?;
                statistics.increment_processed();
            }
            Err(error) => {
                errors.push(format!(
                    "Failed to process file {}: {}",
                    path.file_name().to_string_lossy(),
                    error
                ));
                statistics.increment_failed();
                statistics.increment_processed();
            }
        }
    }

    send_if_needed(&mut emails, true)?;

    for error in errors {
        error!("{error}");
    }
    Ok(())
}

fn read_eml_to_new_email(path: &PathBuf) -> Result<EmailNew> {
    if !path.is_file() {
        return Err(anyhow!("No such file : {:?}", path));
    }

    let eml_bytes = fs::read(path).context("Could not read eml to string")?;

    let email = mailparse::parse_mail(&eml_bytes)?;

    let read_header_as_string = |header_name: &str| -> Result<String> {
        match parse_header(&email.headers, header_name) {
            Some(id) => {
                if id.is_empty() {
                    Err(anyhow!("{} header blank", header_name))
                } else {
                    Ok(id)
                }
            }
            None => Err(anyhow!("Could not read {} header", header_name)),
        }
    };

    let message_id = read_header_as_string("Message-Id")?;
    let date_str = read_header_as_string("Date")?;
    let timestamp = DateTime::parse_from_rfc2822(&date_str)?.with_timezone(&Utc);

    // Get Attachments
    let mut attachments = Vec::new();

    for part in email.subparts {
        let content_disposition = part.get_content_disposition();
        if content_disposition.disposition == DispositionType::Attachment {
            let get_param = |param_name: &str| -> Result<&String> {
                content_disposition
                    .params
                    .get(param_name)
                    .ok_or(anyhow!("Could not get attachment param: {}", param_name))
            };

            let attachment_filename = get_param("filename")?;
            let size: u64 = get_param("size")?.parse()?;
            let extension = Path::new(attachment_filename)
                .extension()
                .context("Could not get attachment extension")?;
            attachments.push(Attachment {
                name: attachment_filename.to_owned(),
                size: size as i32,
                content_type: format!(".{}", extension.to_string_lossy()),
                attachment_reference: None,
                content_hash: None,
                inline: None,
            });
        }
    }

    // Get File name - for mailbox name
    let file_name = path
        .file_name()
        .context("Could not get eml file name")?
        .to_string_lossy()
        .to_string();

    // Get mime content
    let eml_str = std::str::from_utf8(&eml_bytes)?;

    Ok(EmailNew {
        id: message_id,
        mailbox: file_name,
        timestamp: timestamp.to_rfc3339(),
        metadata: None,
        attachments: Some(attachments),
        mime_content: eml_str.to_string(),
    })
}
pub fn parse_header(headers: &[MailHeader], header: &str) -> Option<String> {
    headers
        .get_first_value(header)
        .map(|value| value.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_read_eml_to_document() {
        let expected_id =
            "<AM9PR02MB66424EB36E9581626499575190DEA@AM9PR02MB6642.eurprd02.prod.outlook.com>";

        let expected_mailbox = "test.eml";
        let expected_timestamp = DateTime::parse_from_rfc2822("Wed, 25 Oct 2023 17:03:22 +0000")
            .unwrap()
            .with_timezone(&Utc);
        let expected_attachments = vec![
            Attachment {
                name: "hello.txt".to_string(),
                size: 176,
                content_type: ".txt".to_string(),
                attachment_reference: None,
                content_hash: None,
                inline: None,
            },
            Attachment {
                name: "world.pdf".to_string(),
                size: 7476,
                content_type: ".pdf".to_string(),
                attachment_reference: None,
                content_hash: None,
                inline: None,
            },
        ];
        let expected_mime_content = include_str!("../../../tests/samples/test.eml");

        let expected_email = EmailNew {
            id: expected_id.to_string(),
            attachments: Some(expected_attachments),
            timestamp: expected_timestamp.to_rfc3339(),
            metadata: None,
            mailbox: expected_mailbox.to_string(),
            mime_content: expected_mime_content.to_string(),
        };

        let actual_email = read_eml_to_new_email(&PathBuf::from("tests/samples/test.eml"))
            .expect("Failed to read eml");

        assert_eq!(expected_email, actual_email);
    }
}
