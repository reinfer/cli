//! ModelVersion utility for CLI commands
//!
//! This module provides a `ModelVersion` wrapper around i32 to provide type safety
//! and easy conversion for model version handling.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

/// A model version wrapper that provides type safety and easy conversion
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModelVersion(pub i32);

impl ModelVersion {
    /// Create a new ModelVersion from an integer
    pub fn new(version: i32) -> Self {
        Self(version)
    }

    /// Get the inner integer
    pub fn as_i32(&self) -> i32 {
        self.0
    }

    /// Convert into the inner integer
    pub fn into_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for ModelVersion {
    fn from(version: i32) -> Self {
        Self(version)
    }
}

impl From<ModelVersion> for i32 {
    fn from(model_version: ModelVersion) -> Self {
        model_version.0
    }
}

impl Display for ModelVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ModelVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.parse::<i32>() {
            Ok(version) => Ok(ModelVersion(version)),
            Err(e) => Err(anyhow!("Could not parse '{}' as model version: {}", s, e)),
        }
    }
}

impl std::ops::Deref for ModelVersion {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_version_creation() {
        let version1 = ModelVersion::new(42);
        let version2 = ModelVersion::from(42);
        let version3: ModelVersion = 42.into();

        assert_eq!(version1, version2);
        assert_eq!(version2, version3);
        assert_eq!(version1.as_i32(), 42);
    }

    #[test]
    fn test_conversions() {
        let version = ModelVersion::new(123);
        let int: i32 = version.clone().into();
        assert_eq!(int, 123);

        let back_to_version: ModelVersion = int.into();
        assert_eq!(version, back_to_version);
    }

    #[test]
    fn test_display_and_fromstr() {
        let version = ModelVersion::new(456);
        let string = version.to_string();
        assert_eq!(string, "456");

        let parsed: ModelVersion = string.parse().unwrap();
        assert_eq!(version, parsed);
    }
}
