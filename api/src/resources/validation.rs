use crate::{LabelGroup, LabelName, ModelVersion};
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModelScore(pub f32);

impl std::fmt::Display for ModelScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetQuality {
    None,
    Poor,
    Average,
    Good,
    Excellent,
}

impl std::fmt::Display for DatasetQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "None",
                Self::Poor => "Poor",
                Self::Average => "Average",
                Self::Good => "Good",
                Self::Excellent => "Excellent",
            }
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModelRating {
    pub score: ModelScore,
    pub quality: DatasetQuality,
}

#[derive(Clone, Deserialize)]
pub struct ValidationSummary {
    pub version: ModelVersion,
    pub model_rating: ModelRating,
    pub reviewed_size: usize,
}

#[derive(Clone, Deserialize)]
pub struct ValidationResponse {
    pub label_groups: Vec<LabelGroup>,
    pub validation: ValidationSummary,
}

impl ValidationResponse {
    pub fn get_default_label_group(&self) -> Option<&LabelGroup> {
        self.label_groups
            .iter()
            .find(|group| group.name.0 == "default")
    }
}
