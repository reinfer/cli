use serde::{Deserialize, Serialize};

/// Corrected ModelConfig that matches the API definition exactly
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelConfig {
    Cm,
    DocPathIxp,
    GptIxp(openapi::models::GptIxpModelConfig),
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self::Cm
    }
}
