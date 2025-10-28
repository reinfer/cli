//! Conversion utilities for OpenAPI types
//!
//! This module provides conversion functions and traits for converting between
//! different OpenAPI generated types, particularly for comments, entities, and annotations.

pub mod comments;
pub mod entities;
pub mod moon_forms;
pub mod names;
pub mod spans;

use openapi::models::{
    AnnotatedComment, EntitiesNew, GroupLabellingsRequest, LabellingGroup, MoonFormGroup,
    MoonFormGroupUpdate,
};

/// Trait for checking if a value has annotations
pub trait HasAnnotations {
    fn has_annotations(&self) -> bool;
}

impl<T> HasAnnotations for Option<T>
where
    T: HasAnnotations,
{
    fn has_annotations(&self) -> bool {
        self.as_ref().is_some_and(|t| t.has_annotations())
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
impl HasAnnotations for GroupLabellingsRequest {
    fn has_annotations(&self) -> bool {
        self.assigned.is_some() || self.dismissed.is_some() || self.uninformative.is_some()
    }
}

impl HasAnnotations for EntitiesNew {
    fn has_annotations(&self) -> bool {
        // Check if entities has any content - this is a simplified check
        // You may need to adjust based on the actual structure of EntitiesNew
        true // For now, assume it has annotations if it exists
    }
}

impl HasAnnotations for MoonFormGroupUpdate {
    fn has_annotations(&self) -> bool {
        !self.assigned.is_empty() || self.drafted.is_some() || !self.dismissed.is_empty()
    }
}

impl HasAnnotations for AnnotatedComment {
    fn has_annotations(&self) -> bool {
        // Check if the comment has any annotations (labels, entities, moon forms)
        self.labelling.iter().any(|l| l.has_annotations())
            || self.entities.iter().any(|e| e.has_annotations())
            || self.moon_forms.iter().any(|m| m.has_annotations())
    }
}

impl HasAnnotations for LabellingGroup {
    fn has_annotations(&self) -> bool {
        // A labelling group has annotations if it has any assigned, dismissed, or uninformative labels
        !self.assigned.is_empty() || !self.dismissed.is_empty() || self.uninformative.is_some()
    }
}

impl HasAnnotations for openapi::models::Entities {
    fn has_annotations(&self) -> bool {
        // Check if entities has any content
        !self.assigned.is_empty() || !self.dismissed.is_empty()
    }
}

impl HasAnnotations for MoonFormGroup {
    fn has_annotations(&self) -> bool {
        // Check if moon form group has any assignments or drafts
        !self.assigned.is_empty() || self.drafted.is_some() || !self.dismissed.is_empty()
    }
}

/// Helper function to determine if an optional vector should be skipped during serialization
pub fn should_skip_serializing_optional_vec<T>(vec: &Option<Vec<T>>) -> bool {
    vec.as_ref().is_none_or(|v| v.is_empty())
}

// Re-export only the conversion functions that are used elsewhere
pub use comments::{convert_annotated_comment, convert_language_string_to_enum};
pub use entities::convert_entities;
pub use moon_forms::convert_moon_form_group;
pub use names::convert_boxed_name_to_label_name;
