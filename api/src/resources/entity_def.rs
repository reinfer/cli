use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EntityDef {
    pub color: u32,
    pub id: Id,
    pub inherits_from: Vec<Id>,
    pub name: Name,
    pub title: String,
    pub trainable: bool,
    #[serde(default)]
    #[serde(rename = "_entity_def_flags")]
    pub entity_def_flags: Vec<EntityDefFlag>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub enum EntityDefFlag {
    AppearsVerbatim,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewEntityDef {
    pub inherits_from: Vec<Id>,
    pub name: Name,
    pub title: String,
    pub trainable: bool,
    #[serde(rename = "_entity_def_flags")]
    #[serde(default)]
    pub entity_def_flags: Vec<EntityDefFlag>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GeneralFieldDef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_type_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_type_name: Option<Name>,

    pub api_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewGeneralFieldDef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_type_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_type_name: Option<Name>,

    pub api_name: String,
}
