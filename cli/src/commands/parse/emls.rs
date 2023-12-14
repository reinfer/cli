use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use log::error;
use mailparse::{DispositionType, MailHeader, MailHeaderMap};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::commands::{
    ensure_uip_user_consents_to_ai_unit_charge,
    parse::{get_files_in_directory, get_progress_bar, Statistics, UPLOAD_BATCH_SIZE},
};
use reinfer_client::{resources::email::AttachmentMetadata, BucketIdentifier, Client, NewEmail};
use structopt::StructOpt;

use super::upload_batch_of_new_emails;

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

pub fn parse(client: &Client, args: &ParseEmlArgs) -> Result<()> {
    let ParseEmlArgs {
        directory,
        bucket,
        no_charge,
        yes,
    } = args;

    if !no_charge && !yes {
        ensure_uip_user_consents_to_ai_unit_charge(client.base_url())?;
    }

    let eml_paths = get_files_in_directory(directory, "eml")?;
    let statistics = Arc::new(Statistics::new());
    let _progress = get_progress_bar(eml_paths.len() as u64, &statistics);

    let bucket = client
        .get_bucket(bucket.clone())
        .with_context(|| format!("Unable to get bucket {}", args.bucket))?;

    let mut emails = Vec::new();
    let mut errors = Vec::new();

    let send = |emails: &mut Vec<NewEmail>| -> Result<()> {
        upload_batch_of_new_emails(client, &bucket.full_name(), emails, *no_charge, &statistics)?;
        emails.clear();
        Ok(())
    };

    for path in eml_paths {
        match read_eml_to_new_email(&path.path()) {
            Ok(new_email) => {
                emails.push(new_email);

                if emails.len() >= UPLOAD_BATCH_SIZE {
                    send(&mut emails)?;
                }
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

    send(&mut emails)?;

    for error in errors {
        error!("{}", error);
    }
    Ok(())
}

fn read_eml_to_new_email(path: &PathBuf) -> Result<NewEmail> {
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
            attachments.push(AttachmentMetadata {
                name: attachment_filename.to_owned(),
                size,
                content_type: format!(".{}", extension.to_string_lossy()),
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

    Ok(NewEmail {
        id: reinfer_client::EmailId(message_id),
        mailbox: reinfer_client::Mailbox(file_name),
        timestamp,
        metadata: None,
        attachments,
        mime_content: reinfer_client::MimeContent(eml_str.to_string()),
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
            AttachmentMetadata {
                name: "hello.txt".to_string(),
                size: 176,
                content_type: ".txt".to_string(),
            },
            AttachmentMetadata {
                name: "world.pdf".to_string(),
                size: 7476,
                content_type: ".pdf".to_string(),
            },
        ];
        let expected_mime_content = include_str!("../../../tests/samples/test.eml");

        let expected_email = NewEmail {
            id: reinfer_client::EmailId(expected_id.to_string()),
            attachments: expected_attachments,
            timestamp: expected_timestamp,
            metadata: None,
            mailbox: reinfer_client::Mailbox(expected_mailbox.to_string()),
            mime_content: reinfer_client::MimeContent(expected_mime_content.to_string()),
        };

        let actual_email = read_eml_to_new_email(&PathBuf::from("tests/samples/test.eml"))
            .expect("Failed to read eml");

        assert_eq!(expected_email, actual_email);
    }
}
