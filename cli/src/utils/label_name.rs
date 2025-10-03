//! LabelName utility for CLI commands
//! 
//! This module provides a `LabelName` wrapper around String to replace
//! the broken OpenAPI generated Name type and provide type safety.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A label name wrapper that provides type safety and easy conversion
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct LabelName(pub String);

impl LabelName {
    /// Create a new LabelName from a string
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner string
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for LabelName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for LabelName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl From<LabelName> for String {
    fn from(label_name: LabelName) -> Self {
        label_name.0
    }
}

impl AsRef<str> for LabelName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for LabelName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for LabelName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_name_creation() {
        let label1 = LabelName::new("test_label");
        let label2 = LabelName::from("test_label");
        let label3: LabelName = "test_label".into();
        
        assert_eq!(label1, label2);
        assert_eq!(label2, label3);
        assert_eq!(label1.as_str(), "test_label");
    }

    #[test]
    fn test_conversions() {
        let label = LabelName::new("test_label");
        let string: String = label.clone().into();
        assert_eq!(string, "test_label");
        
        let back_to_label: LabelName = string.into();
        assert_eq!(label, back_to_label);
    }
}
