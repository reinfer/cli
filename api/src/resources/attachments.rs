use std::path::Path;

use crate::AttachmentReference;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ContentHash(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UploadAttachmentResponse {
    pub content_hash: ContentHash,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AttachmentMetadata {
    pub name: String,
    pub size: u64,
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_reference: Option<AttachmentReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<ContentHash>,
}

impl AttachmentMetadata {
    pub fn extension(&self) -> Option<String> {
        let path = Path::new(&self.name);
        path.extension()
            .map(|path| path.to_string_lossy().to_string())
    }
}
