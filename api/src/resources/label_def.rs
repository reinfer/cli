use crate::resources::comment::should_skip_serializing_optional_vec;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pretrained: Option<NewLabelDefPretrained>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trainable: Option<bool>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub moon_form: Option<Vec<MoonFormFieldDef>>,
}

impl TryFrom<&LabelDef> for NewLabelDef {
    type Error = crate::Error;
    fn try_from(value: &LabelDef) -> Result<Self, Self::Error> {
        let label_as_string = json!(value).to_string();

        let result: NewLabelDef =
            serde_json::from_str(&label_as_string).map_err(|err| crate::Error::Unknown {
                message: "Could not parse new label def".to_string(),
                source: Box::new(err),
            })?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LabelDefProperty {
    pub name: String,
    pub value: f64,
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

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CreateOrUpdateLabelDefsBulkRequest {
    pub label_defs: Vec<NewLabelDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CreateOrUpdateLabelDefsBulkResponse {}
