use anyhow::{anyhow, Result};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

/// A validated comment ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommentId(pub String);

impl FromStr for CommentId {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.is_empty() {
            return Err(anyhow!("Comment ID cannot be empty"));
        }
        Ok(CommentId(string.to_owned()))
    }
}

impl Display for CommentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for CommentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<CommentId> for String {
    fn from(id: CommentId) -> Self {
        id.0
    }
}

/// A validated source ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceId(pub String);

impl FromStr for SourceId {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.is_empty() {
            return Err(anyhow!("Source ID cannot be empty"));
        }
        Ok(SourceId(string.to_owned()))
    }
}

impl Display for SourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SourceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<SourceId> for String {
    fn from(id: SourceId) -> Self {
        id.0
    }
}
