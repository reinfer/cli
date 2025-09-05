//! Utility functions for comment processing that were previously in the legacy client

use serde::{Deserialize, Serialize};

/// Trait for checking if a value has annotations
pub trait HasAnnotations {
    fn has_annotations(&self) -> bool;
}

impl<T> HasAnnotations for Option<T>
where
    T: HasAnnotations,
{
    fn has_annotations(&self) -> bool {
        self.as_ref().map_or(false, |t| t.has_annotations())
    }
}

impl<T> HasAnnotations for Vec<T>
where
    T: HasAnnotations,
{
    fn has_annotations(&self) -> bool {
        !self.is_empty() && self.iter().any(|item| item.has_annotations())
    }
}

// Implement HasAnnotations for OpenAPI types
impl HasAnnotations for openapi::models::GroupLabellingsRequest {
    fn has_annotations(&self) -> bool {
        self.assigned.is_some() || self.dismissed.is_some() || self.uninformative.is_some()
    }
}

impl HasAnnotations for openapi::models::EntitiesNew {
    fn has_annotations(&self) -> bool {
        // Check if entities has any content - this is a simplified check
        // You may need to adjust based on the actual structure of EntitiesNew
        true // For now, assume it has annotations if it exists
    }
}

impl HasAnnotations for openapi::models::MoonFormGroupUpdate {
    fn has_annotations(&self) -> bool {
        !self.assigned.is_empty() || self.drafted.is_some() || self.dismissed.is_some()
    }
}

/// Helper function to determine if an optional vector should be skipped during serialization
pub fn should_skip_serializing_optional_vec<T>(vec: &Option<Vec<T>>) -> bool {
    vec.as_ref().map_or(true, |v| v.is_empty())
}
