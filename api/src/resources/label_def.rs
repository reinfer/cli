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

impl LabelDef {
    pub fn to_new(&self) -> NewLabelDef<'_> {
        NewLabelDef {
            name: &self.name,
            description: Some(&self.description),
            external_id: self.external_id.as_deref(),
            pretrained: self
                .pretrained
                .as_ref()
                .map(|pretrained| pretrained.to_new()),
            title: Some(&self.title),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewLabelDef<'request> {
    pub name: &'request Name,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<&'request str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pretrained: Option<NewLabelDefPretrained<'request>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LabelDefPretrained {
    pub id: PretrainedId,
    pub name: Name,
}

impl LabelDefPretrained {
    pub fn to_new(&self) -> NewLabelDefPretrained<'_> {
        NewLabelDefPretrained {
            id: &self.id,
            name: Some(&self.name),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewLabelDefPretrained<'request> {
    pub id: &'request PretrainedId,
    pub name: Option<&'request Name>,
}
