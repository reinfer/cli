//! Name utilities for CLI commands
//!
//! This module provides type-safe wrappers for different kinds of names used in the CLI:
//! - `FullName`: Resource names in "owner/name" format

use anyhow::{anyhow, Result};
// Removed unused serde imports since LabelName is deleted
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Deref;
use std::str::FromStr;

/// A full name in "owner/name" format (for resources like datasets, sources, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullName(pub String);

impl FromStr for FullName {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.split('/').count() == 2 {
            Ok(FullName(s.to_owned()))
        } else {
            Err(anyhow!("expected owner/name, got '{s}'"))
        }
    }
}

impl FullName {
    pub fn owner(&self) -> &str {
        self.0.split('/').next().unwrap()
    }
    pub fn name(&self) -> &str {
        self.0.split('/').nth(1).unwrap()
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for FullName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl Deref for FullName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// LabelName was removed - just use String directly instead of pointless wrapper

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_name() {
        let full_name = FullName::from_str("owner/dataset").unwrap();
        assert_eq!(full_name.owner(), "owner");
        assert_eq!(full_name.name(), "dataset");
        assert_eq!(full_name.as_str(), "owner/dataset");
    }

    // LabelName tests removed - just use String directly
}
