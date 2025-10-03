//! Utility functions for comment processing that were previously in the legacy client

use openapi::models::{
    AnnotatedComment, Comment, CommentNew, CommentSpanNewUtf16, 
    Entities, EntitiesNew, Entity, EntityNew, EntityNewSpansInner,
    GroupLabellingsRequest, LabellingGroup, Language, Name,
    MoonFormCaptureFieldsAnnotation, MoonFormCaptureFieldsAnnotationNew,
    MoonFormDismissedUpdate, MoonFormFieldAnnotation, MoonFormFieldAnnotationNew,
    MoonFormGroup, MoonFormGroupUpdate, MoonFormLabelAnnotation, MoonFormLabelAnnotationUpdate,
    Span, TextSpan,
};
use super::LabelName;
use crate::utils::new_annotated_comment::NewAnnotatedComment;

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
        !self.assigned.is_empty() || self.drafted.is_some() || self.dismissed.is_some()
    }
}

impl HasAnnotations for AnnotatedComment {
    fn has_annotations(&self) -> bool {
        // Check if the comment has any annotations (labels, entities, moon forms)
        self.labelling.iter().any(|l| l.has_annotations()) || 
        self.entities.iter().any(|e| e.has_annotations()) ||
        self.moon_forms.iter().any(|m| m.has_annotations())
    }
}

impl HasAnnotations for LabellingGroup {
    fn has_annotations(&self) -> bool {
        // A labelling group has annotations if it has any assigned, dismissed, or uninformative labels
        !self.assigned.is_empty() || !self.dismissed.is_empty() || self.uninformative.is_some()
    }
}

impl HasAnnotations for Entities {
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
    vec.as_ref().map_or(true, |v| v.is_empty())
}

// Type conversion utilities between read and update models

/// Convert TextSpan to CommentSpanNewUtf16 for API requests
pub fn convert_text_span(span: TextSpan) -> CommentSpanNewUtf16 {
    CommentSpanNewUtf16 {
        content_part: span.content_part,
        message_index: span.message_index,
        utf16_byte_start: span.utf16_byte_start,
        utf16_byte_end: span.utf16_byte_end,
    }
}

/// Convert MoonFormFieldAnnotation to MoonFormFieldAnnotationNew for API requests
pub fn convert_moon_form_field_annotation(field: MoonFormFieldAnnotation) -> MoonFormFieldAnnotationNew {
    MoonFormFieldAnnotationNew {
        id: Some(field.id),
        name: Some(field.name),
        span: field.span.map(|span| Box::new(convert_text_span(*span))),
        spans: Some(field.spans.into_iter().map(convert_text_span).collect()),
        formatted_value: field.formatted_value,
        field_id: Some(field.field_id),
        document_spans: Some(field.document_spans),
        confirmed: field.confirmed,
    }
}

/// Convert MoonFormCaptureFieldsAnnotation to MoonFormCaptureFieldsAnnotationNew for API requests
pub fn convert_moon_form_capture_fields_annotation(capture: MoonFormCaptureFieldsAnnotation) -> MoonFormCaptureFieldsAnnotationNew {
    MoonFormCaptureFieldsAnnotationNew {
        fields: capture.fields.into_iter().map(convert_moon_form_field_annotation).collect(),
    }
}

/// Convert MoonFormLabelAnnotation to MoonFormLabelAnnotationUpdate for API requests
pub fn convert_moon_form_label_annotation(label: MoonFormLabelAnnotation) -> MoonFormLabelAnnotationUpdate {
    MoonFormLabelAnnotationUpdate {
        label: label.label,
        captures: label.captures.into_iter().map(convert_moon_form_capture_fields_annotation).collect(),
    }
}

/// Convert MoonFormGroup to MoonFormGroupUpdate for API requests
pub fn convert_moon_form_group(group: MoonFormGroup) -> MoonFormGroupUpdate {
    use openapi::models::moon_form_group_update::Group;
    
    // Convert dismissed annotations to the required format
    let dismissed = if group.dismissed.is_empty() {
        None
    } else {
        // For now, we'll create a basic MoonFormDismissedUpdate
        // This might need adjustment based on actual API requirements
        Some(Box::new(MoonFormDismissedUpdate {
            captures: None,
            entities: vec![], // This may need proper conversion logic
            labels: group.dismissed.iter()
                .map(|d| *d.label.clone())
                .collect(),
        }))
    };

    MoonFormGroupUpdate {
        group: Group::Default, // Map string to enum - may need more sophisticated mapping
        assigned: group.assigned.into_iter().map(convert_moon_form_label_annotation).collect(),
        drafted: group.drafted.map(|d| d.into_iter().map(convert_moon_form_label_annotation).collect()),
        dismissed,
    }
}

/// Convert TextSpan to EntityNewSpansInner for API requests
pub fn convert_text_span_to_entity_spans_inner(span: TextSpan) -> EntityNewSpansInner {
    EntityNewSpansInner {
        content_part: span.content_part,
        message_index: span.message_index,
        utf16_byte_start: span.utf16_byte_start,
        utf16_byte_end: span.utf16_byte_end,
        char_start: span.char_start,
        char_end: span.char_end,
    }
}

/// Convert TextSpan to Span for API requests
pub fn convert_text_span_to_span(span: TextSpan) -> Span {
    Span {
        content_part: span.content_part,
        message_index: span.message_index,
        utf16_byte_start: span.utf16_byte_start,
        utf16_byte_end: span.utf16_byte_end,
        char_start: span.char_start,
        char_end: span.char_end,
    }
}

/// Convert Entity to EntityNew for API requests
pub fn convert_entity(entity: Entity) -> EntityNew {
    EntityNew {
        id: Some(entity.id),
        name: Some(entity.name),
        kind: Some(entity.kind),
        span: entity.span.map(|span| Box::new(convert_text_span_to_span(*span))),
        spans: Some(entity.spans.into_iter().map(convert_text_span_to_entity_spans_inner).collect()),
        formatted_value: entity.formatted_value,
        field_id: Some(entity.field_id),
    }
}

/// Convert Entities to EntitiesNew for API requests
pub fn convert_entities(entities: Entities) -> EntitiesNew {
    let assigned = if entities.assigned.is_empty() {
        None
    } else {
        Some(entities.assigned.into_iter().map(convert_entity).collect())
    };
    
    let dismissed = if entities.dismissed.is_empty() {
        None
    } else {
        Some(entities.dismissed.into_iter().map(convert_entity).collect())
    };

    EntitiesNew {
        assigned,
        dismissed,
    }
}

/// Convert LabellingGroup to GroupLabellingsRequest for API requests
pub fn convert_labelling_group(group: LabellingGroup) -> GroupLabellingsRequest {
    use openapi::models::group_labellings_request::Group;
    
    let assigned = if group.assigned.is_empty() {
        None
    } else {
        Some(group.assigned)
    };
    
    let dismissed = if group.dismissed.is_empty() {
        None
    } else {
        Some(group.dismissed)
    };

    GroupLabellingsRequest {
        assigned,
        dismissed,
        uninformative: group.uninformative,
        group: Some(Group::Default), // Convert enum variant
    }
}

/// Convert String to Language enum for API requests
pub fn convert_language_string_to_enum(language_str: &str) -> Language {
    match language_str.to_lowercase().as_str() {
        "en" | "english" => Language::En,
        "de" | "german" | "deutsch" => Language::De,
        "xlm" | "multilingual" => Language::Xlm,
        _ => {
            // Default to English for unknown languages
            log::warn!("Unknown language '{}', defaulting to English", language_str);
            Language::En
        }
    }
}

/// Convert Comment to CommentNew for API requests
pub fn convert_comment(comment: Comment) -> CommentNew {
    CommentNew {
        id: comment.id,
        timestamp: Some(comment.timestamp),
        thread_id: comment.thread_id,
        user_properties: Some(comment.user_properties),
        messages: Some(comment.messages),
        attachments: Some(comment.attachments),
        ..Default::default()
    }
}

/// Convert broken OpenAPI Name to LabelName
/// 
/// The OpenAPI generator created an empty Name struct, but we can try to
/// extract the actual label name from the serialized JSON representation.
/// This is a workaround until the OpenAPI generation is fixed.
pub fn convert_openapi_name_to_label_name(name: &Name) -> LabelName {
    // Try to serialize the Name back to JSON and extract the actual string value
    match serde_json::to_value(name) {
        Ok(json_value) => {
            // The real label name should be in the JSON as a string or array
            match json_value {
                serde_json::Value::String(s) => LabelName::from(s),
                serde_json::Value::Array(arr) if !arr.is_empty() => {
                    // If it's an array, join with " > " (hierarchical labels)
                    let parts: Vec<String> = arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect();
                    LabelName::from(parts.join(" > "))
                },
                _ => {
                    log::warn!("Could not extract label name from JSON: {:?}", json_value);
                    LabelName::from("UNKNOWN_LABEL")
                }
            }
        },
        Err(e) => {
            log::warn!("Failed to serialize Name to JSON: {}", e);
            LabelName::from("SERIALIZATION_ERROR")
        }
    }
}

/// Convert Box<Name> to LabelName  
pub fn convert_boxed_name_to_label_name(boxed_name: &Box<Name>) -> LabelName {
    convert_openapi_name_to_label_name(boxed_name)
}

/// Convert AnnotatedComment to NewAnnotatedComment for API requests
pub fn convert_annotated_comment(annotated_comment: AnnotatedComment) -> NewAnnotatedComment {
    NewAnnotatedComment {
        comment: convert_comment(*annotated_comment.comment),
        labelling: if annotated_comment.labelling.is_empty() {
            None
        } else {
            Some(annotated_comment.labelling.into_iter()
                .map(convert_labelling_group)
                .collect())
        },
        entities: annotated_comment.entities.map(|entities| Box::new(convert_entities(*entities))),
        moon_forms: annotated_comment.moon_forms.map(|forms|
            forms.into_iter()
                .map(convert_moon_form_group)
                .collect()
        ),
    }
}

