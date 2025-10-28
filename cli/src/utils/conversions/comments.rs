//! Comment conversion utilities

use super::entities::convert_entities;
use super::moon_forms::convert_moon_form_group;
use crate::utils::new_annotated_comment::NewAnnotatedComment;
use openapi::models::{
    AnnotatedComment, Comment, CommentNew, GroupLabellingsRequest, LabellingGroup, Language,
};

/// Convert String to Language enum for API requests
pub fn convert_language_string_to_enum(language_str: &str) -> Language {
    match language_str.to_lowercase().as_str() {
        "en" | "english" => Language::En,
        "de" | "german" | "deutsch" => Language::De,
        "xlm" | "multilingual" => Language::Xlm,
        _ => {
            // Default to English for unknown languages
            log::warn!("Unknown language '{language_str}', defaulting to English");
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

/// Convert AnnotatedComment to NewAnnotatedComment for API requests
pub fn convert_annotated_comment(annotated_comment: AnnotatedComment) -> NewAnnotatedComment {
    NewAnnotatedComment {
        comment: convert_comment(*annotated_comment.comment),
        labelling: if annotated_comment.labelling.is_empty() {
            None
        } else {
            Some(
                annotated_comment
                    .labelling
                    .into_iter()
                    .map(convert_labelling_group)
                    .collect(),
            )
        },
        entities: annotated_comment
            .entities
            .map(|entities| Box::new(convert_entities(*entities))),
        moon_forms: annotated_comment
            .moon_forms
            .map(|forms| forms.into_iter().map(convert_moon_form_group).collect()),
    }
}
