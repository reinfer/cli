use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ContentHash(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UploadAttachmentResponse {
    pub content_hash: ContentHash,
}
