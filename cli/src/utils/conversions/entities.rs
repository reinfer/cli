//! Entity conversion utilities

use super::spans::{convert_text_span_to_entity_spans_inner, convert_text_span_to_span};
use openapi::models::{Entities, EntitiesNew, Entity, EntityNew};

/// Convert Entity to EntityNew for API requests
pub fn convert_entity(entity: Entity) -> EntityNew {
    EntityNew {
        id: Some(entity.id),
        name: Some(entity.name),
        kind: Some(entity.kind),
        span: entity
            .span
            .map(|span| Box::new(convert_text_span_to_span(*span))),
        spans: Some(
            entity
                .spans
                .into_iter()
                .map(convert_text_span_to_entity_spans_inner)
                .collect(),
        ),
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
