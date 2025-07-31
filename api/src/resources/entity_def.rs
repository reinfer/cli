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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<EntityDefRuleSet>,
    #[serde(default)]
    pub instructions: Option<String>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct EntityDefRuleSet {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressors: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<FieldChoiceApi>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<RegexPattern>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct FieldChoiceApi {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RegexPattern {
    EntityTemplatesProperty(EntityTemplatesProperty),
    LegacyEntityPatten(LegacyEntityPattern),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LegacyEntityPattern {
    pub pattern: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EntityTemplatesProperty {
    templates: Vec<EntityTemplateProperty>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EntityTemplateProperty {
    template: String,
    pattern: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<EntityRuleSetNew>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct EntityRuleSetNew {
    #[serde(rename = "suppressors", skip_serializing_if = "Vec::is_empty")]
    pub suppressors: Vec<String>,
    #[serde(rename = "choices", skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<FieldChoiceNew>,
    #[serde(rename = "regex")]
    pub regex: Option<RegexPattern>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct FieldChoiceNew {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "values")]
    pub values: Vec<String>,
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
