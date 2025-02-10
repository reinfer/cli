use crate::Error;
use std::str::FromStr;

use crate::{ReducibleResponse, SplittableRequest};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::resources::attachments::AttachmentMetadata;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Mailbox(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct MimeContent(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

impl FromStr for Id {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(Self(string.to_owned()))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
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
    pub conversation_topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_read: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub received_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Email {
    pub id: Id,
    pub mailbox: Mailbox,
    pub timestamp: DateTime<Utc>,
    pub mime_content: MimeContent,
    pub metadata: Option<EmailMetadata>,
    pub attachments: Vec<AttachmentMetadata>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewEmail {
    pub id: Id,
    pub mailbox: Mailbox,
    pub timestamp: DateTime<Utc>,
    pub mime_content: MimeContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<EmailMetadata>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<AttachmentMetadata>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct PutEmailsRequest {
    pub emails: Vec<NewEmail>,
}

impl SplittableRequest for PutEmailsRequest {
    fn split(self) -> impl Iterator<Item = Self>
    where
        Self: Sized,
    {
        self.emails.into_iter().map(|email| Self {
            emails: vec![email],
        })
    }

    fn count(&self) -> usize {
        self.emails.len()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PutEmailsResponse {}

impl ReducibleResponse for PutEmailsResponse {}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Continuation(pub String);

#[derive(Debug, Clone, Deserialize)]
pub struct EmailsIterPage {
    pub emails: Vec<Email>,
    pub continuation: Option<Continuation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetEmailResponse {
    pub emails: Vec<Email>,
}
