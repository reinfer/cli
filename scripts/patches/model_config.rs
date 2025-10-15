use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelConfig {
    Cm(models::CmModelConfig),
    DocPathIxp(models::DocPathIxpModelConfig),
    GptIxp(models::GptIxpModelConfig),
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self::Cm(models::CmModelConfig::default())
    }
}
