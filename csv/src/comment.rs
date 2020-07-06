use chrono::{DateTime, Utc};
use serde::{
    ser::{SerializeMap, Serializer},
    Serialize,
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Comment {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "PropertyMap::is_empty", default)]
    pub user_properties: PropertyMap,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Message {
    pub body: MessageBody,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<MessageSubject>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MessageBody {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MessageSubject {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PropertyMap(HashMap<String, PropertyValue>);

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    String(String),
    Number(f64),
}

impl Deref for PropertyMap {
    type Target = HashMap<String, PropertyValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PropertyMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PropertyMap {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        PropertyMap(HashMap::with_capacity(capacity))
    }

    #[inline]
    pub fn insert_number(&mut self, key: String, value: f64) {
        self.0.insert(key, PropertyValue::Number(value));
    }

    #[inline]
    pub fn insert_string(&mut self, key: String, value: String) {
        self.0.insert(key, PropertyValue::String(value));
    }

    // Provided despite deref, for `skip_serializing_if`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

const STRING_PROPERTY_PREFIX: &str = "string:";
const NUMBER_PROPERTY_PREFIX: &str = "number:";

impl Serialize for PropertyMap {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_map(Some(self.len()))?;
        if self.0.is_empty() {
            return state.end();
        }

        let mut full_name = String::with_capacity(32);
        for (key, value) in &self.0 {
            full_name.clear();
            match *value {
                PropertyValue::String(ref value) => {
                    if !value.trim().is_empty() {
                        full_name.push_str(STRING_PROPERTY_PREFIX);
                        full_name.push_str(key);
                        state.serialize_entry(&full_name, &value)?;
                    }
                }
                PropertyValue::Number(value) => {
                    full_name.push_str(NUMBER_PROPERTY_PREFIX);
                    full_name.push_str(key);
                    state.serialize_entry(&full_name, &value)?;
                }
            }
        }
        state.end()
    }
}
