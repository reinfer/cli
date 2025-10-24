use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserPropertyName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Number(NotNan<f64>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PropertyFilter {
    #[serde(skip_serializing_if = "<[_]>::is_empty", default)]
    pub one_of: Vec<PropertyValue>,
    #[serde(skip_serializing_if = "<[_]>::is_empty", default)]
    pub not_one_of: Vec<PropertyValue>,
    #[serde(skip_serializing_if = "<[_]>::is_empty", default)]
    pub domain_not_one_of: Vec<PropertyValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<NotNan<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<NotNan<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPropertiesFilter(pub HashMap<UserPropertyName, PropertyFilter>);
