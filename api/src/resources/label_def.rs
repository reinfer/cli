use crate::resources::comment::should_skip_serializing_optional_vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct PretrainedId(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LabelDef {
    pub name: Name,

    #[serde(default)]
    pub instructions: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pretrained: Option<LabelDefPretrained>,
    #[serde(default)]
    pub title: String,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub moon_form: Option<Vec<MoonFormFieldDef>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewLabelDef {
    pub name: Name,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pretrained: Option<NewLabelDefPretrained>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub moon_form: Option<Vec<MoonFormFieldDef>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LabelDefPretrained {
    pub id: PretrainedId,
    pub name: Name,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewLabelDefPretrained {
    pub id: PretrainedId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Name>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MoonFormFieldDef {
    pub name: String,
    pub kind: String,
}
