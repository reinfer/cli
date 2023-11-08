use crate::{LabelGroup, LabelName};
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LabelValidation {
    pub thresholds: Vec<NotNan<f64>>,
    pub precisions: Vec<NotNan<f64>>,
    pub recalls: Vec<NotNan<f64>>,
}

#[derive(Serialize)]
pub struct LabelValidationRequest {
    pub label: LabelName,
}

#[derive(Deserialize)]
pub struct LabelValidationResponse {
    pub label_validation: LabelValidation,
}

#[derive(Clone, Deserialize)]
pub struct ValidationResponse {
    pub label_groups: Vec<LabelGroup>,
}

impl ValidationResponse {
    pub fn get_default_label_group(&self) -> Option<&LabelGroup> {
        self.label_groups
            .iter()
            .find(|group| group.name.0 == "default")
    }
}
