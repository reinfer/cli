use serde::{Deserialize, Serialize};

/// Enum for attribute filter types, similar to the reinfer_client AttributeFilterEnum
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AttributeFilterEnum {
    StringAnyOf {
        any_of: Vec<String>,
    },
    NumberRange {
        minimum: Option<usize>,
        maximum: Option<usize>,
    },
}
