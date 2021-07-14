use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct PretrainedId(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LabelDef {
    pub name: Name,

    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pretrained: Option<LabelDefPretrained>,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewLabelDef {
    pub name: Name,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pretrained: Option<NewLabelDefPretrained>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
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
