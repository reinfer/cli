//! Stream full name utility for CLI commands
//!
//! This module provides a `StreamFullName` struct that represents a stream identifier
//! in the format 'owner/dataset/stream'.

use anyhow::Result;
use std::str::FromStr;

/// Represents a stream full name in the format 'owner/dataset/stream'
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamFullName {
    pub owner: String,
    pub dataset: String,
    pub stream: String,
}

impl FromStr for StreamFullName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!(
                "Stream name must be in format 'owner/dataset/stream', got: '{}'",
                s
            ));
        }

        Ok(StreamFullName {
            owner: parts[0].to_string(),
            dataset: parts[1].to_string(),
            stream: parts[2].to_string(),
        })
    }
}

impl std::fmt::Display for StreamFullName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.owner, self.dataset, self.stream)
    }
}

impl StreamFullName {
    /// Get the owner part of the stream full name
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Get the dataset part of the stream full name
    pub fn dataset(&self) -> &str {
        &self.dataset
    }

    /// Get the stream part of the stream full name
    pub fn stream(&self) -> &str {
        &self.stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_stream_name() {
        let stream = "owner/dataset/stream".parse::<StreamFullName>().unwrap();
        assert_eq!(stream.owner(), "owner");
        assert_eq!(stream.dataset(), "dataset");
        assert_eq!(stream.stream(), "stream");
    }

    #[test]
    fn test_parse_invalid_stream_name() {
        assert!("owner/dataset".parse::<StreamFullName>().is_err());
        assert!("owner/dataset/stream/extra"
            .parse::<StreamFullName>()
            .is_err());
    }

    #[test]
    fn test_display() {
        let stream = StreamFullName {
            owner: "owner".to_string(),
            dataset: "dataset".to_string(),
            stream: "stream".to_string(),
        };
        assert_eq!(format!("{stream}"), "owner/dataset/stream");
    }
}
