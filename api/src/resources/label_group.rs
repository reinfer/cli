use serde::{Deserialize, Serialize};

use crate::resources::label_def::{LabelDef, NewLabelDef};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

use once_cell::sync::Lazy;

pub static DEFAULT_LABEL_GROUP_NAME: Lazy<Name> = Lazy::new(|| Name("default".to_owned()));

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LabelGroup {
    pub name: Name,

    #[serde(default)]
    pub label_defs: Vec<LabelDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewLabelGroup {
    pub name: Name,

    #[serde(default)]
    pub label_defs: Vec<NewLabelDef>,
}
