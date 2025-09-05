//! Generic resource identifier utility for CLI commands
//! 
//! This module provides a generic `ResourceIdentifier` enum that can represent any resource
//! either by its ID (hex string) or by its full name (owner/name format).
//! 
//! # Examples
//! 
//! ```rust
//! use std::str::FromStr;
//! use crate::utils::resource_identifier::ResourceIdentifier;
//! use crate::utils::full_name::FullName;
//! 
//! // Parse by ID
//! let id_resource = ResourceIdentifier::from_str("abc123def456").unwrap();
//! assert!(matches!(id_resource, ResourceIdentifier::Id(_)));
//! 
//! // Parse by full name
//! let name_resource = ResourceIdentifier::from_str("owner/resource-name").unwrap();
//! assert!(matches!(name_resource, ResourceIdentifier::FullName(_)));
//! assert_eq!(name_resource.owner(), Some("owner"));
//! assert_eq!(name_resource.name(), Some("resource-name"));
//! ```

use crate::utils::full_name::FullName;
use anyhow::Result;
use std::str::FromStr;

/// Represents a resource identifier that can be either an ID or a full name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceIdentifier {
    /// Resource identified by its hex ID
    Id(String),
    /// Resource identified by its full name in owner/name format
    FullName(FullName),
}

impl FromStr for ResourceIdentifier {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        if s.contains('/') {
            Ok(ResourceIdentifier::FullName(FullName::from_str(s)?))
        } else {
            Ok(ResourceIdentifier::Id(s.to_owned()))
        }
    }
}

impl ResourceIdentifier {
    /// Extract the owner from a FullName resource identifier
    pub fn owner(&self) -> Option<&str> {
        match self {
            ResourceIdentifier::FullName(full_name) => Some(full_name.owner()),
            ResourceIdentifier::Id(_) => None,
        }
    }

    /// Extract the name from a FullName resource identifier
    pub fn name(&self) -> Option<&str> {
        match self {
            ResourceIdentifier::FullName(full_name) => Some(full_name.name()),
            ResourceIdentifier::Id(_) => None,
        }
    }

    /// Extract the ID from an ID resource identifier
    pub fn id(&self) -> Option<&str> {
        match self {
            ResourceIdentifier::Id(id) => Some(id),
            ResourceIdentifier::FullName(_) => None,
        }
    }

    /// Get the full name if this is a FullName resource identifier
    pub fn full_name(&self) -> Option<&FullName> {
        match self {
            ResourceIdentifier::FullName(full_name) => Some(full_name),
            ResourceIdentifier::Id(_) => None,
        }
    }
}

impl std::fmt::Display for ResourceIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceIdentifier::Id(id) => write!(f, "{}", id),
            ResourceIdentifier::FullName(full_name) => write!(f, "{}", full_name),
        }
    }
}

// Type aliases for backward compatibility and clarity
pub type SourceIdentifier = ResourceIdentifier;
pub type DatasetIdentifier = ResourceIdentifier;
pub type BucketIdentifier = ResourceIdentifier;
pub type UserIdentifier = ResourceIdentifier;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_id() {
        let id = "abc123".parse::<ResourceIdentifier>().unwrap();
        assert!(matches!(id, ResourceIdentifier::Id(_)));
        assert_eq!(id.id(), Some("abc123"));
        assert_eq!(id.owner(), None);
        assert_eq!(id.name(), None);
    }

    #[test]
    fn test_parse_full_name() {
        let full_name = "owner/resource".parse::<ResourceIdentifier>().unwrap();
        assert!(matches!(full_name, ResourceIdentifier::FullName(_)));
        assert_eq!(full_name.owner(), Some("owner"));
        assert_eq!(full_name.name(), Some("resource"));
        assert_eq!(full_name.id(), None);
    }

    #[test]
    fn test_display() {
        let id = ResourceIdentifier::Id("abc123".to_string());
        assert_eq!(format!("{}", id), "abc123");

        let full_name = ResourceIdentifier::FullName(FullName::from_str("owner/resource").unwrap());
        assert_eq!(format!("{}", full_name), "owner/resource");
    }

    #[test]
    fn test_type_aliases() {
        let source_id: SourceIdentifier = "abc123".parse().unwrap();
        let dataset_id: DatasetIdentifier = "owner/dataset".parse().unwrap();
        let bucket_id: BucketIdentifier = "def456".parse().unwrap();

        assert!(matches!(source_id, ResourceIdentifier::Id(_)));
        assert!(matches!(dataset_id, ResourceIdentifier::FullName(_)));
        assert!(matches!(bucket_id, ResourceIdentifier::Id(_)));
    }
}
