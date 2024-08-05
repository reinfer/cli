use crate::{CommentId, NewComment, PropertyMap, TransformTag};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

use super::attachments::AttachmentMetadata;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub raw_email: RawEmail,
    pub user_properties: PropertyMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<CommentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawEmail {
    pub body: RawEmailBody,
    pub headers: RawEmailHeaders,
    pub attachments: Vec<AttachmentMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawEmailBody {
    Plain(String),
    Html(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawEmailHeaders {
    Raw(String),
    Parsed(ParsedHeaders),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedHeaders(#[serde(serialize_with = "ordered_map")] pub HashMap<String, String>);

fn ordered_map<S>(value: &HashMap<String, String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SyncRawEmailsRequest<'request> {
    pub documents: &'request [Document],
    pub include_comments: bool,
    pub transform_tag: &'request TransformTag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRawEmailsResponse {
    pub new: usize,
    pub updated: usize,
    pub unchanged: usize,
    #[serde(default)]
    pub comments: Vec<NewComment>,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_deserialize_with_comment_id() {
        let document = Document {
            comment_id: Some(CommentId("abc123".to_string())),
            raw_email: RawEmail {
                attachments: vec![],
                body: RawEmailBody::Plain("Hello world".to_string()),
                headers: RawEmailHeaders::Raw(
                    r#"Subject: This is the subject
To: user@example.com
From: sender@example.com"#
                        .to_string(),
                ),
            },
            user_properties: PropertyMap::new(),
        };

        let expected_document = "{\"raw_email\":{\"body\":{\"plain\":\"Hello world\"},\"headers\":{\"raw\":\"Subject: This is the subject\\nTo: user@example.com\\nFrom: sender@example.com\"},\"attachments\":[]},\"user_properties\":{},\"comment_id\":\"abc123\"}";

        assert_eq!(
            serde_json::to_string(&document).expect("Document serialization error"),
            expected_document
        )
    }

    #[test]
    pub fn test_document_serialize_plain_body_raw_headers() {
        let document = Document {
            comment_id: None,
            raw_email: RawEmail {
                attachments: vec![
                    AttachmentMetadata {
                        name: "hello.pdf".to_string(),
                        size: 1000,
                        content_type: "pdf".to_string(),
                        attachment_reference: None,
                        content_hash: None,
                    },
                    AttachmentMetadata {
                        name: "world.csv".to_string(),
                        size: 9999,
                        content_type: "csv".to_string(),
                        attachment_reference: None,
                        content_hash: None,
                    },
                ],
                body: RawEmailBody::Plain("Hello world".to_string()),
                headers: RawEmailHeaders::Raw(
                    r#"Subject: This is the subject
To: user@example.com
From: sender@example.com"#
                        .to_string(),
                ),
            },
            user_properties: PropertyMap::new(),
        };

        let expected_document = "{\"raw_email\":{\"body\":{\"plain\":\"Hello world\"},\"headers\":{\"raw\":\"Subject: This is the subject\\nTo: user@example.com\\nFrom: sender@example.com\"},\"attachments\":[{\"name\":\"hello.pdf\",\"size\":1000,\"content_type\":\"pdf\"},{\"name\":\"world.csv\",\"size\":9999,\"content_type\":\"csv\"}]},\"user_properties\":{}}";

        assert_eq!(
            serde_json::to_string(&document).expect("Document serialization error"),
            expected_document
        )
    }

    #[test]
    pub fn test_document_serialize_html_body_parsed_headers() {
        let parsed_headers = ParsedHeaders(HashMap::from([
            (
                "Date".to_string(),
                "Thu, 09 Jan 2020 16:34:45 +0000".to_string(),
            ),
            ("From".to_string(), "alice@company.com".to_string()),
            ("Message-ID".to_string(), "abcdef@company.com".to_string()),
            (
                "References".to_string(),
                "<01234@company.com> <56789@company.com>".to_string(),
            ),
            ("Subject".to_string(), "Figures Request".to_string()),
            ("To".to_string(), "bob@organisation.org".to_string()),
        ]));

        let document = Document {
            comment_id: None,
            raw_email: RawEmail {
                body: RawEmailBody::Html("<b>Hello world</b>".to_string()),
                headers: RawEmailHeaders::Parsed(parsed_headers),
                attachments: Vec::new(),
            },
            user_properties: PropertyMap::new(),
        };

        assert_eq!(
            serde_json::to_string(&document).expect("Document serialization error"),
            "{\"raw_email\":{\"body\":{\"html\":\"<b>Hello world</b>\"},\"headers\":{\"parsed\":{\"Date\":\"Thu, 09 Jan 2020 16:34:45 +0000\",\"From\":\"alice@company.com\",\"Message-ID\":\"abcdef@company.com\",\"References\":\"<01234@company.com> <56789@company.com>\",\"Subject\":\"Figures Request\",\"To\":\"bob@organisation.org\"}},\"attachments\":[]},\"user_properties\":{}}"
        )
    }
}
