//! New annotated comment utility for CLI commands
//!
//! This module provides a `NewAnnotatedComment` struct that represents a comment
//! with annotations using OpenAPI generated types.

use openapi::models;

/// A comment with annotations using OpenAPI types
#[derive(Debug, Clone)]
pub struct NewAnnotatedComment {
    /// The base comment data - used structurally in API calls
    #[allow(dead_code)]
    pub comment: models::CommentNew,
    pub labelling: Option<Vec<models::GroupLabellingsRequest>>,
    pub entities: Option<Box<models::EntitiesNew>>,
    pub moon_forms: Option<Vec<models::MoonFormGroupUpdate>>,
}

impl NewAnnotatedComment {
    pub fn has_annotations(&self) -> bool {
        self.labelling.is_some() || self.entities.is_some() || self.moon_forms.is_some()
    }
}
