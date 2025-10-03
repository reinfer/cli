//! Utility functions for attachment processing

use std::path::Path;
use openapi::models::Attachment;

/// Extension trait for Attachment to add utility methods
pub trait AttachmentExt {
    /// Extract the file extension from the attachment name
    fn extension(&self) -> Option<String>;
}

impl AttachmentExt for Attachment {
    fn extension(&self) -> Option<String> {
        let path = Path::new(&self.name);
        path.extension()
            .map(|ext| ext.to_string_lossy().to_string())
    }
}
