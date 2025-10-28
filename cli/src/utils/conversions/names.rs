//! Name conversion utilities

use openapi::models::Name;

/// Convert broken OpenAPI Name to String
///
/// The OpenAPI generator created an empty Name struct, but we can try to
/// extract the actual label name from the serialized JSON representation.
/// This is a workaround until the OpenAPI generation is fixed.
pub fn convert_openapi_name_to_label_name(name: &Name) -> String {
    // Try to serialize the Name back to JSON and extract the actual string value
    match serde_json::to_value(name) {
        Ok(json_value) => {
            // The real label name should be in the JSON as a string or array
            match json_value {
                serde_json::Value::String(s) => s,
                serde_json::Value::Array(arr) if !arr.is_empty() => {
                    // If it's an array, join with " > " (hierarchical labels)
                    let parts: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect();
                    parts.join(" > ")
                }
                _ => {
                    log::warn!("Could not extract label name from JSON: {json_value:?}");
                    "UNKNOWN_LABEL".to_string()
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to serialize Name to JSON: {e}");
            "SERIALIZATION_ERROR".to_string()
        }
    }
}

/// Convert Box<Name> to String
pub fn convert_boxed_name_to_label_name(boxed_name: &Name) -> String {
    convert_openapi_name_to_label_name(boxed_name)
}
