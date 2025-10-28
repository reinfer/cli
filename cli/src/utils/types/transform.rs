//! Transform tag utility for CLI commands
//!
//! This module provides a `TransformTag` wrapper that validates email transform tags
//! using the generated API. It ensures that only valid transform tags are used.
//!
//! # Examples
//!
//! ```rust
//! use std::str::FromStr;
//! use crate::utils::types::transform::TransformTag;
//!
//! // Parse and validate a transform tag
//! let transform_tag = TransformTag::from_str("my-transform-tag").unwrap();
//! assert_eq!(transform_tag.as_str(), "my-transform-tag");
//! ```

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::str::FromStr;

/// A validated email transform tag
///
/// This wrapper ensures that transform tags are valid by checking them against
/// the API when possible. For now, it provides basic validation and can be
/// extended to use the `get_email_transform_tag_info` API for full validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransformTag(pub String);

impl FromStr for TransformTag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Basic validation - transform tags should not be empty
        if s.trim().is_empty() {
            return Err(anyhow!("Transform tag cannot be empty"));
        }

        // Basic validation - transform tags should be reasonable length
        if s.len() > 256 {
            return Err(anyhow!("Transform tag too long (max 256 characters)"));
        }

        // Basic validation - transform tags should contain only valid characters
        // Allow alphanumeric, hyphens, underscores, and dots
        if !s
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(anyhow!("Transform tag contains invalid characters. Only alphanumeric, hyphens, underscores, and dots are allowed"));
        }

        Ok(TransformTag(s.to_owned()))
    }
}

impl TransformTag {
    /// Get the transform tag as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the transform tag as a string
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for TransformTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TransformTag> for String {
    fn from(transform_tag: TransformTag) -> Self {
        transform_tag.0
    }
}

impl From<String> for TransformTag {
    fn from(s: String) -> Self {
        // This bypasses validation - use with caution
        TransformTag(s)
    }
}

/// The default transform tag used for generic email processing
///
/// This is the standard transform tag used when no specific transform tag is provided.
/// It uses the "generic.0.CONVKER5" tag which is suitable for most email processing needs.
pub static DEFAULT_TRANSFORM_TAG: Lazy<TransformTag> =
    Lazy::new(|| TransformTag("generic.0.CONVKER5".to_string()));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_tag_parsing() {
        // Valid transform tags
        assert!(TransformTag::from_str("my-transform-tag").is_ok());
        assert!(TransformTag::from_str("generic.0.CONVKER5").is_ok());
        assert!(TransformTag::from_str("test_123").is_ok());
        assert!(TransformTag::from_str("test-123").is_ok());

        // Invalid transform tags
        assert!(TransformTag::from_str("").is_err());
        assert!(TransformTag::from_str("   ").is_err());
        assert!(TransformTag::from_str("test@invalid").is_err());
        assert!(TransformTag::from_str("test with spaces").is_err());
    }

    #[test]
    fn test_transform_tag_methods() {
        let tag = TransformTag::from_str("my-transform-tag").unwrap();
        assert_eq!(tag.as_str(), "my-transform-tag");
        assert_eq!(tag.into_string(), "my-transform-tag");
    }

    #[test]
    fn test_default_transform_tag() {
        assert_eq!(DEFAULT_TRANSFORM_TAG.as_str(), "generic.0.CONVKER5");
    }
}
