// Standard library imports
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

// External crate imports
use anyhow::{anyhow, Context, Result};
use cfb::CompoundFile;
use colored::Colorize;
use log::error;
use once_cell::sync::Lazy;
use regex::Regex;
use structopt::StructOpt;

// OpenAPI imports
use openapi::{
    apis::{
        configuration::Configuration,
        sources_api::{get_source, get_source_by_id},
    },
    models::{Attachment, Headers, RawEmail, RawEmailBody, RawEmailDocument},
};

// Local crate imports
use crate::{
    commands::{ensure_uip_user_consents_to_ai_unit_charge, DEFAULT_TRANSFORM_TAG},
    parse::{get_files_in_directory, upload_batch_of_documents, Statistics},
    progress::{Options as ProgressOptions, Progress},
    utils::{types::identifiers::SourceIdentifier, types::transform::TransformTag},
};
const MSG_NAME_USER_PROPERTY_NAME: &str = "MSG NAME ID";
const STREAM_PATH_ATTACHMENT_STORE_PREFIX: &str = "__attach_version1.0_#";
const UPLOAD_BATCH_SIZE: usize = 128;

static CONTENT_TYPE_MIME_HEADER_RX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Content-Type:((\s)+.+\n)+").unwrap());
static CONTENT_TRANSFER_ENCODING_MIME_HEADER_RX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Content-Transfer-Encoding:((\s)+.+\n)+").unwrap());
static STREAM_PATH_MESSAGE_BODY_PLAIN: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_1000001F"));
static STREAM_PATH_MESSAGE_HEADER: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_007d001F"));
static STREAM_PATH_ATTACHMENT_FILENAME: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_3707001F"));
static STREAM_PATH_ATTACHMENT_EXTENSION: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_3703001F"));
static STREAM_PATH_ATTACHMENT_DATA: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_37010102"));

/// Command-line arguments for parsing MSG files
#[derive(Debug, StructOpt)]
pub struct ParseMsgArgs {
    #[structopt(short = "d", long = "dir", parse(from_os_str))]
    /// Directory containing the MSG files
    directory: PathBuf,

    #[structopt(short = "s", long = "source")]
    /// Source name or ID to upload emails to
    source: SourceIdentifier,

    #[structopt(long = "transform-tag")]
    /// Transform tag to use for processing
    transform_tag: Option<TransformTag>,

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,

    #[structopt(short = "y", long = "yes")]
    /// Consent to AI unit charge. Suppresses confirmation prompt
    yes: bool,
}

/// Reads raw binary data from a stream within the compound file
fn read_stream(stream_path: &Path, compound_file: &mut CompoundFile<File>) -> Result<Vec<u8>> {
    let mut stream = compound_file.open_stream(stream_path)?;
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Reads a UTF-16LE encoded string from a stream within the compound file
fn read_unicode_stream_to_string(
    stream_path: &Path,
    compound_file: &mut CompoundFile<File>,
) -> Result<String> {
    if !compound_file.is_stream(stream_path) {
        return Err(anyhow!(
            "Could not find stream {}. Please check that you are using Unicode MSG files",
            stream_path.to_string_lossy()
        ));
    }

    // Stream data is a UTF-16LE string encoded as Vec<u8>
    let data = read_stream(stream_path, compound_file)?;
    Ok(utf16le_stream_to_string(&data))
}

/// Decodes a UTF-16LE data stream from raw bytes
fn utf16le_stream_to_string(data: &[u8]) -> String {
    let mut decoder = encoding_rs::UTF_16LE.new_decoder();
    let mut buffer = String::with_capacity(data.len());

    loop {
        let (coder_result, _, _) = decoder.decode_to_string(data, &mut buffer, true);
        use encoding_rs::CoderResult;
        match coder_result {
            CoderResult::OutputFull => buffer.reserve(data.len()),
            CoderResult::InputEmpty => return buffer,
        }
    }
}

/// Generates the path to an attachment store within the MSG file
fn get_attachment_store_path(attachment_number: usize) -> PathBuf {
    PathBuf::from(format!(
        "{STREAM_PATH_ATTACHMENT_STORE_PREFIX}{attachment_number:08}"
    ))
}

/// Reads attachment metadata from the MSG file
fn read_attachment(
    attachment_path: PathBuf,
    compound_file: &mut CompoundFile<File>,
) -> Result<Attachment> {
    let mut attachment_name_path = attachment_path.clone();
    attachment_name_path.push(&*STREAM_PATH_ATTACHMENT_FILENAME);

    let mut content_type_path = attachment_path.clone();
    content_type_path.push(&*STREAM_PATH_ATTACHMENT_EXTENSION);

    let mut data_path = attachment_path.clone();
    data_path.push(&*STREAM_PATH_ATTACHMENT_DATA);

    let name = read_unicode_stream_to_string(&attachment_name_path, compound_file)
        .context("Failed to read attachment filename")?;
    let content_type = read_unicode_stream_to_string(&content_type_path, compound_file)
        .context("Failed to read attachment content type")?;
    let data = read_stream(&data_path, compound_file).context("Failed to read attachment data")?;

    Ok(Attachment {
        name,
        content_type,
        size: data.len() as i32,
        attachment_reference: None,
        content_hash: None,
        inline: None,
    })
}

/// Removes content-type and content-transfer-encoding headers from email headers
fn remove_content_headers(headers_string: String) -> Result<String> {
    let clean_headers_string = CONTENT_TYPE_MIME_HEADER_RX
        .replace(&headers_string, "")
        .to_string();

    let clean_headers_string = CONTENT_TRANSFER_ENCODING_MIME_HEADER_RX
        .replace(&clean_headers_string, "")
        .to_string();

    Ok(clean_headers_string)
}

fn read_msg_to_raw_email_document(path: &PathBuf) -> Result<RawEmailDocument> {
    if !path.is_file() {
        return Err(anyhow!("No such file: {:?}", path));
    }

    let mut compound_file = cfb::open(path)?;

    // Headers
    let headers_string =
        read_unicode_stream_to_string(&STREAM_PATH_MESSAGE_HEADER, &mut compound_file)?;

    // As the content type won't match the parsed value from the body in the msg
    let headers_string_no_content_headers = remove_content_headers(headers_string)?;

    let plain_body_string =
        read_unicode_stream_to_string(&STREAM_PATH_MESSAGE_BODY_PLAIN, &mut compound_file)?;

    // Attachments
    let mut attachment_number = 0;
    let mut attachments = Vec::new();
    loop {
        let attachment_path = get_attachment_store_path(attachment_number);

        if compound_file.is_storage(&attachment_path) {
            attachments.push(read_attachment(attachment_path, &mut compound_file)?);
        } else {
            break;
        }

        attachment_number += 1;
    }

    // User Properties
    let mut user_properties = HashMap::new();
    user_properties.insert(
        MSG_NAME_USER_PROPERTY_NAME.to_string(),
        path.file_name()
            .context("Could not get file name")?
            .to_string_lossy()
            .to_string(),
    );

    Ok(RawEmailDocument {
        raw_email: Box::new(RawEmail {
            headers: Box::new(Headers {
                raw: headers_string_no_content_headers,
                parsed: std::collections::HashMap::new(),
            }),
            body: Box::new(RawEmailBody {
                plain: Some(plain_body_string),
                html: None,
            }),
            attachments: if attachments.is_empty() {
                None
            } else {
                Some(attachments)
            },
        }),
        user_properties: Some(user_properties),
        comment_id: None,
    })
}

/// Parses MSG files from a directory and uploads them to the specified source
pub fn parse(config: &Configuration, args: &ParseMsgArgs) -> Result<()> {
    let ParseMsgArgs {
        directory,
        source,
        transform_tag,
        no_charge,
        yes,
    } = args;

    // Check for user consent if charging is enabled
    if !no_charge && !yes {
        ensure_uip_user_consents_to_ai_unit_charge(&config.base_path.parse()?)?;
    }

    // Get list of MSG files to process
    let msg_paths = get_files_in_directory(directory, "msg", true)
        .with_context(|| format!("Failed to scan directory: {}", directory.display()))?;

    if msg_paths.is_empty() {
        return Err(anyhow!(
            "No MSG files found in directory: {}",
            directory.display()
        ));
    }
    let statistics = Arc::new(Statistics::new());
    let _progress = get_progress_bar(msg_paths.len() as u64, &statistics);
    // Resolve source using the API
    let source = match source {
        SourceIdentifier::Id(source_id) => {
            let response = get_source_by_id(config, source_id)
                .with_context(|| format!("Failed to get source by ID: {source_id}"))?;
            response.source
        }
        SourceIdentifier::FullName(full_name) => {
            let response = get_source(config, full_name.owner(), full_name.name())
                .with_context(|| format!("Failed to get source: {full_name}"))?;
            response.source
        }
    };
    let transform_tag = transform_tag
        .clone()
        .unwrap_or_else(|| DEFAULT_TRANSFORM_TAG.clone());

    let mut documents = Vec::<RawEmailDocument>::new();
    let mut errors = Vec::new();

    // Helper function to upload a batch of documents
    let send_batch = |documents: &mut Vec<RawEmailDocument>| -> Result<()> {
        if documents.is_empty() {
            return Ok(());
        }

        upload_batch_of_documents(
            config,
            &source,
            documents,
            &transform_tag,
            *no_charge,
            &statistics,
        )?;
        documents.clear();
        Ok(())
    };

    for path in msg_paths {
        match read_msg_to_raw_email_document(&path.path()) {
            Ok(document) => {
                documents.push(document);

                if documents.len() >= UPLOAD_BATCH_SIZE {
                    send_batch(&mut documents)?;
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

    send_batch(&mut documents)?;

    for error in errors {
        error!("{error}");
    }

    Ok(())
}

/// Creates a progress bar for tracking MSG file processing
fn get_progress_bar(total_bytes: u64, statistics: &Arc<Statistics>) -> Progress {
    Progress::new(
        move |statistic| {
            let num_processed = statistic.num_processed();
            let num_failed = statistic.num_failed();
            let num_uploaded = statistic.num_uploaded();
            (
                num_processed as u64,
                format!(
                    "{} {} {} {} {} {}",
                    num_processed.to_string().bold(),
                    "processed".dimmed(),
                    num_failed.to_string().bold(),
                    "failed".dimmed(),
                    num_uploaded.to_string().bold(),
                    "uploaded".dimmed()
                ),
            )
        },
        statistics,
        Some(total_bytes),
        ProgressOptions { bytes_units: false },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_read_msg_to_raw_email_document_non_unicode() {
        let result =
            read_msg_to_raw_email_document(&PathBuf::from("tests/samples/non-unicode.msg"));

        assert_eq!(result.expect_err("Expected Error Result").to_string(), "Could not find stream __substg1.0_007d001F. Please check that you are using Unicode MSG files");
    }

    #[test]
    fn test_read_msg_to_raw_email_document_unicode() {
        let mut expected_user_properties = HashMap::new();
        expected_user_properties.insert("MSG NAME ID".to_string(), "unicode.msg".to_string());

        let expected_headers = "Received: from DB8PR02MB5883.eurprd02.prod.outlook.com (2603:10a6:10:116::17)\r\n by AM6PR02MB4215.eurprd02.prod.outlook.com with HTTPS; Wed, 25 Oct 2023\r\n 17:03:35 +0000\r\nAuthentication-Results: dkim=none (message not signed)\r\n header.d=none;dmarc=none action=none header.from=uipath.com;\r\nReceived: from AM9PR02MB6642.eurprd02.prod.outlook.com (2603:10a6:20b:2d2::18)\r\n by DB8PR02MB5883.eurprd02.prod.outlook.com (2603:10a6:10:116::17) with\r\n Microsoft SMTP Server (version=TLS1_2,\r\n cipher=TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384) id 15.20.6907.33; Wed, 25 Oct\r\n 2023 17:03:23 +0000\r\nReceived: from AM9PR02MB6642.eurprd02.prod.outlook.com\r\n ([fe80::fb6e:4d16:d9ba:33ae]) by AM9PR02MB6642.eurprd02.prod.outlook.com\r\n ([fe80::fb6e:4d16:d9ba:33ae%6]) with mapi id 15.20.6933.019; Wed, 25 Oct 2023\r\n 17:03:23 +0000\r\nFrom: Joe Prosser <joe.prosser@uipath.com>\r\nTo: Andra Buica <andra.buica@uipath.com>\r\nSubject: Re: Testing the CLI!!\r\nThread-Topic: Testing the CLI!!\r\nThread-Index: AQHaB2N76HBc3H6u4Eeu9hAZAXjmE7BauHazgAAAIR2AAAJRsw==\r\nDate: Wed, 25 Oct 2023 17:03:22 +0000\r\nMessage-ID:\r\n <AM9PR02MB66424EB36E9581626499575190DEA@AM9PR02MB6642.eurprd02.prod.outlook.com>\r\nReferences:\r\n <AM9PR02MB664276669FA004F9D7AE85F890DEA@AM9PR02MB6642.eurprd02.prod.outlook.com>\r\n <AM6PR02MB4215248605797A7CCC598DA28EDEA@AM6PR02MB4215.eurprd02.prod.outlook.com>\r\n <AM9PR02MB664263ADFB43C2AA252F510F90DEA@AM9PR02MB6642.eurprd02.prod.outlook.com>\r\nIn-Reply-To:\r\n <AM9PR02MB664263ADFB43C2AA252F510F90DEA@AM9PR02MB6642.eurprd02.prod.outlook.com>\r\nAccept-Language: en-GB, en-US\r\nContent-Language: en-GB\r\nX-MS-Has-Attach: yes\r\nX-MS-Exchange-Organization-SCL: 1\r\nX-MS-TNEF-Correlator:\r\n <AM9PR02MB66424EB36E9581626499575190DEA@AM9PR02MB6642.eurprd02.prod.outlook.com>\r\nmsip_labels:\r\nMIME-Version: 1.0\r\nX-MS-Exchange-Organization-MessageDirectionality: Originating\r\nX-MS-Exchange-Organization-AuthSource: AM9PR02MB6642.eurprd02.prod.outlook.com\r\nX-MS-Exchange-Organization-AuthAs: Internal\r\nX-MS-Exchange-Organization-AuthMechanism: 04\r\nX-MS-Exchange-Organization-Network-Message-Id:\r\n 178907c2-2871-4221-a75a-08dbd57c4bf2\r\nX-MS-PublicTrafficType: Email\r\nX-MS-TrafficTypeDiagnostic:\r\n AM9PR02MB6642:EE_|DB8PR02MB5883:EE_|AM6PR02MB4215:EE_\r\nReturn-Path: joe.prosser@uipath.com\r\nX-MS-Exchange-Organization-ExpirationStartTime: 25 Oct 2023 17:03:23.2098\r\n (UTC)\r\nX-MS-Exchange-Organization-ExpirationStartTimeReason: OriginalSubmit\r\nX-MS-Exchange-Organization-ExpirationInterval: 1:00:00:00.0000000\r\nX-MS-Exchange-Organization-ExpirationIntervalReason: OriginalSubmit\r\nX-MS-Office365-Filtering-Correlation-Id: 178907c2-2871-4221-a75a-08dbd57c4bf2\r\nX-MS-Exchange-AtpMessageProperties: SA|SL\r\nX-Microsoft-Antispam: BCL:0;\r\nX-Forefront-Antispam-Report:\r\n CIP:255.255.255.255;CTRY:;LANG:en;SCL:1;SRV:;IPV:NLI;SFV:NSPM;H:AM9PR02MB6642.eurprd02.prod.outlook.com;PTR:;CAT:NONE;SFS:;DIR:INT;\r\nX-MS-Exchange-CrossTenant-OriginalArrivalTime: 25 Oct 2023 17:03:22.9433\r\n (UTC)\r\nX-MS-Exchange-CrossTenant-FromEntityHeader: Hosted\r\nX-MS-Exchange-CrossTenant-Id: d8353d2a-b153-4d17-8827-902c51f72357\r\nX-MS-Exchange-CrossTenant-AuthSource: AM9PR02MB6642.eurprd02.prod.outlook.com\r\nX-MS-Exchange-CrossTenant-AuthAs: Internal\r\nX-MS-Exchange-CrossTenant-Network-Message-Id: 178907c2-2871-4221-a75a-08dbd57c4bf2\r\nX-MS-Exchange-CrossTenant-MailboxType: HOSTED\r\nX-MS-Exchange-CrossTenant-UserPrincipalName: ++FDvUGfWQ/f3Ycjz3vJlna939o3V0zI8K8BPWCQ7aaQDhAJYe5XMf5Tr6B6e/5ucWri5/sUCnvrb1GDymXZmA==\r\nX-MS-Exchange-Transport-CrossTenantHeadersStamped: DB8PR02MB5883\r\nX-MS-Exchange-Transport-EndToEndLatency: 00:00:12.3444456\r\nX-MS-Exchange-Processed-By-BccFoldering: 15.20.6907.032\r\nX-Microsoft-Antispam-Mailbox-Delivery:\r\n\tucf:0;jmr:0;auth:0;dest:I;ENG:(910001)(944506478)(944626604)(920097)(425001)(930097)(140003)(1420103);\r\nX-Microsoft-Antispam-Message-Info:\r\n\thxMOcmClDYIwXiLHDHfW3mj4bIuPkLWLYD9z839jTLrsdaQbyDGWXui95ou9iKuUSUHzubxtiSwgj9OVTVInYaJ6SuN1imGY/n99Cn1vAx4VAh1xSYBUsm2bHu5atiwnMZTQaWg7BA03Yj6BSrOYqh49oLJrh9blZmtlkMs4uCi1Y5Y7YpEMpzLi7ye2fg+gPqELrNLHIKfziazCdrPNLKQ+9tlbb6UHbxeX1YVGY2ebnYXPwRilEP++cljm3hV8abvlgC1GF+nsuIS5XJwhkHmgTyetr7iZE3GWR9XC0NsB9w5bQgg82r75ozlGKWVKkuev1IlRjStpKaJbOoPBvo/6SCqSbWaQinyUEfPDXiJOYHW3D0xsf4uCpbFpb1D+TpQr+dFaCxBNdmFjBrd/SCHoyyl9QAO1nz0W5cUAwjSDKN7Pv7iKIUlx24nkDCeYeVqKhNcqulwIlEP6ewBBL2BTSplNPApIbliiHhh/Z6mes1xx3dfB5T4tIUv6wINSH3G2Ddec2fLzeTHknuyaOA9Wj8ks+JIjE+i5/CPidxfH4ACoggwbdnLwwnwniGcNoZKdyM+G4whogPe2oKXPggX9er/44bUOlKEcK5DsPFEZX2xKzsg7JwPPLcgO/lbP2iN/yFww6I6vri27P29a86np21iNSGOb51giFj5wgSq4iZLeRe64cEj4+i4K1KosNLBgDYTj/WGrioHl5Xe9ww==\r\n";

        let expected_body = "Hey, \r\n\r\nWe should check that attachments work to ✅\r\n\r\nJoe \r\n________________________________\r\n\r\nFrom: Joe Prosser <joe.prosser@uipath.com>\r\nSent: 25 October 2023 17:53\r\nTo: Andra Buica <andra.buica@uipath.com>\r\nSubject: Re: Testing the CLI!! \r\n \r\nHey,\r\n\r\nFingers cross  🤞 \r\n\r\nJoe\r\n________________________________\r\n\r\nFrom: Andra Buica <andra.buica@uipath.com>\r\nSent: 25 October 2023 17:52\r\nTo: Joe Prosser <joe.prosser@uipath.com>\r\nSubject: Re: Testing the CLI!! \r\n \r\n\r\nHey, \r\n\r\n \r\n\r\nHopefully it will work! \r\n\r\n \r\n\r\nAndra\r\n\r\n \r\n\r\nFrom: Joe Prosser <joe.prosser@uipath.com>\r\nDate: Wednesday, 25 October 2023 at 17:52\r\nTo: Andra Buica <andra.buica@uipath.com>\r\nSubject: Testing the CLI!!\r\n\r\nHey,\r\n\r\n \r\n\r\nDo you think it will work?\r\n\r\n \r\n\r\nJoe \r\n\r\n";

        let expected_attachments = vec![
            Attachment {
                name: "hello.txt".to_string(),
                size: 12,
                content_type: ".txt".to_string(),
                attachment_reference: None,
                content_hash: None,
                inline: None,
            },
            Attachment {
                name: "world.pdf".to_string(),
                size: 7302,
                content_type: ".pdf".to_string(),
                attachment_reference: None,
                content_hash: None,
                inline: None,
            },
        ];

        let expected_document = RawEmailDocument {
            raw_email: Box::new(RawEmail {
                headers: Box::new(Headers {
                    raw: expected_headers.to_string(),
                    parsed: std::collections::HashMap::new(),
                }),
                body: Box::new(RawEmailBody {
                    plain: Some(expected_body.to_string()),
                    html: None,
                }),
                attachments: Some(expected_attachments),
            }),
            user_properties: Some(expected_user_properties),
            comment_id: None,
        };

        let actual_document =
            read_msg_to_raw_email_document(&PathBuf::from("tests/samples/unicode.msg"))
                .expect("Failed to read msg");

        assert_eq!(expected_document, actual_document);
    }
}
