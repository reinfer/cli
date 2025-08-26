//! Moon form conversion utilities

use super::spans::convert_text_span;
use openapi::models::{
    MoonFormCaptureFieldsAnnotation, MoonFormCaptureFieldsAnnotationNew, MoonFormFieldAnnotation,
    MoonFormFieldAnnotationNew, MoonFormGroup, MoonFormGroupUpdate, MoonFormLabelAnnotation,
    MoonFormLabelAnnotationUpdate,
};

/// Convert MoonFormFieldAnnotation to MoonFormFieldAnnotationNew for API requests
pub fn convert_moon_form_field_annotation(
    field: MoonFormFieldAnnotation,
) -> MoonFormFieldAnnotationNew {
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
pub fn convert_moon_form_capture_fields_annotation(
    capture: MoonFormCaptureFieldsAnnotation,
) -> MoonFormCaptureFieldsAnnotationNew {
    MoonFormCaptureFieldsAnnotationNew {
        fields: capture
            .fields
            .into_iter()
            .map(convert_moon_form_field_annotation)
            .collect(),
    }
}

/// Convert MoonFormLabelAnnotation to MoonFormLabelAnnotationUpdate for API requests
pub fn convert_moon_form_label_annotation(
    label: MoonFormLabelAnnotation,
) -> MoonFormLabelAnnotationUpdate {
    MoonFormLabelAnnotationUpdate {
        label: label.label,
        captures: label
            .captures
            .into_iter()
            .map(convert_moon_form_capture_fields_annotation)
            .collect(),
    }
}

/// Convert MoonFormGroup to MoonFormGroupUpdate for API requests
pub fn convert_moon_form_group(group: MoonFormGroup) -> MoonFormGroupUpdate {
    use openapi::models::moon_form_group_update::Group;

    // Convert dismissed annotations to the required format
    let dismissed: Vec<MoonFormLabelAnnotationUpdate> = group
        .dismissed
        .into_iter()
        .map(convert_moon_form_label_annotation)
        .collect();

    MoonFormGroupUpdate {
        group: Group::Default, // Map string to enum - may need more sophisticated mapping
        assigned: group
            .assigned
            .into_iter()
            .map(convert_moon_form_label_annotation)
            .collect(),
        drafted: group.drafted.map(|d| {
            d.into_iter()
                .map(convert_moon_form_label_annotation)
                .collect()
        }),
        dismissed,
    }
}
