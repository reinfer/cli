use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Mailbox(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct MimeContent(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EmailMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_read: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AttachmentMetadata {
    pub name: String,
    pub size: u64,
    pub content_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewEmail {
    pub id: Id,
    pub mailbox: Mailbox,
    pub timestamp: DateTime<Utc>,
    pub mime_content: MimeContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<EmailMetadata>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub attachments: Vec<AttachmentMetadata>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct PutEmailsRequest<'request> {
    pub emails: &'request [NewEmail],
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PutEmailsResponse {}
