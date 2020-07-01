use crate::errors::Error;
use chrono::{DateTime, Utc};
use serde::{
    de::{Deserializer, Error as SerdeError, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
use serde_json::{json, Value as JsonValue};
use std::str::FromStr;
use std::{
    collections::HashMap,
    fmt::{Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
    result::Result as StdResult,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Uid(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ThreadId(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CommentFilter(pub JsonValue);

impl Default for CommentFilter {
    fn default() -> Self {
        Self::empty()
    }
}

impl CommentFilter {
    pub fn empty() -> Self {
        CommentFilter(json!({}))
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GetRecentRequest<'a> {
    pub limit: usize,
    pub filter: &'a CommentFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<&'a Continuation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Continuation(pub String);

#[derive(Debug, Clone, Deserialize)]
pub struct RecentCommentsPage {
    pub results: Vec<AnnotatedComment>,
    pub continuation: Option<Continuation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct GetLabellingsAfter(pub String);

#[derive(Debug, Clone, Deserialize)]
pub struct GetAnnotationsResponse {
    pub results: Vec<AnnotatedComment>,
    #[serde(default)]
    pub after: Option<GetLabellingsAfter>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateAnnotationsRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelling: Option<&'a NewLabelling>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<&'a NewEntities>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentsIterPage {
    pub comments: Vec<Comment>,
    pub continuation: Option<Continuation>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SyncCommentsRequest<'request> {
    pub comments: &'request [NewComment],
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncCommentsResponse {
    pub new: usize,
    pub updated: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Comment {
    pub id: Id,
    pub uid: Uid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<ThreadId>,
    pub timestamp: DateTime<Utc>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "PropertyMap::is_empty", default)]
    pub user_properties: PropertyMap,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NewComment {
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<ThreadId>,
    pub timestamp: DateTime<Utc>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "PropertyMap::is_empty", default)]
    pub user_properties: PropertyMap,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Message {
    pub body: MessageBody,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<MessageSubject>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<MessageSignature>,

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MessageBody {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MessageSubject {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MessageSignature {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Sentiment {
    #[serde(rename = "positive")]
    Positive,

    #[serde(rename = "negative")]
    Negative,
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

impl<'de> Deserialize<'de> for PropertyMap {
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(PropertyMapVisitor)
    }
}

struct PropertyMapVisitor;
impl<'de> Visitor<'de> for PropertyMapVisitor {
    type Value = PropertyMap;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "a user property map")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<PropertyMap, E> {
        Ok(PropertyMap::new())
    }

    fn visit_map<M>(self, mut access: M) -> StdResult<PropertyMap, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut values = PropertyMap::with_capacity(access.size_hint().unwrap_or(0));

        while let Some(mut key) = access.next_key()? {
            if strip_prefix(&mut key, STRING_PROPERTY_PREFIX) {
                values.insert(key, PropertyValue::String(access.next_value()?));
            } else if strip_prefix(&mut key, NUMBER_PROPERTY_PREFIX) {
                values.insert(key, PropertyValue::Number(access.next_value()?));
            } else {
                return Err(M::Error::custom(format!(
                    "user property full name `{}` has invalid \
                     type prefix",
                    key
                )));
            }
        }

        Ok(values)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnnotatedComment {
    pub comment: Comment,
    #[serde(skip_serializing_if = "should_skip_serializing_labelling")]
    pub labelling: Option<Labelling>,
    #[serde(skip_serializing_if = "should_skip_serializing_entities")]
    pub entities: Option<Entities>,
}

impl AnnotatedComment {
    pub fn has_annotations(&self) -> bool {
        let has_labels = self
            .labelling
            .as_ref()
            .map(|labelling| !labelling.assigned.is_empty() || !labelling.dismissed.is_empty())
            .unwrap_or(false);
        let has_entities = self
            .entities
            .as_ref()
            .map(|entities| !entities.assigned.is_empty() || !entities.dismissed.is_empty())
            .unwrap_or(false);
        has_labels || has_entities
    }

    pub fn without_predictions(mut self) -> Self {
        self.labelling = self.labelling.and_then(|mut labelling| {
            if labelling.assigned.is_empty() && labelling.dismissed.is_empty() {
                None
            } else {
                labelling.predicted = None;
                Some(labelling)
            }
        });
        self.entities = self.entities.and_then(|mut entities| {
            if entities.assigned.is_empty() && entities.dismissed.is_empty() {
                None
            } else {
                entities.predicted = None;
                Some(entities)
            }
        });
        self
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewAnnotatedComment {
    pub comment: NewComment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelling: Option<NewLabelling>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<NewEntities>,
}

impl NewAnnotatedComment {
    pub fn has_annotations(&self) -> bool {
        let has_labels = self
            .labelling
            .as_ref()
            .map(|labelling| !labelling.assigned.is_empty() || !labelling.dismissed.is_empty())
            .unwrap_or(false);
        let has_entities = self
            .entities
            .as_ref()
            .map(|entities| !entities.assigned.is_empty() || !entities.dismissed.is_empty())
            .unwrap_or(false);
        has_labels || has_entities
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Labelling {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub assigned: Vec<Label>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dismissed: Vec<Label>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub predicted: Option<Vec<PredictedLabel>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewLabelling {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub assigned: Vec<Label>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dismissed: Vec<Label>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Label {
    pub name: LabelName,
    pub sentiment: Sentiment,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PredictedLabel {
    pub name: LabelName,
    pub sentiment: f64,
    pub probability: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PredictedLabelParts {
    pub name: Vec<LabelName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentiment: Option<f64>,
    pub probability: f64,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub struct LabelName(pub String);

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Entities {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub assigned: Vec<Entity>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dismissed: Vec<Entity>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub predicted: Option<Vec<Entity>>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct NewEntities {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub assigned: Vec<NewEntity>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dismissed: Vec<NewEntity>,
}

fn should_skip_serializing_optional_vec<T>(maybe_vec: &Option<Vec<T>>) -> bool {
    if let Some(actual_vec) = maybe_vec {
        actual_vec.is_empty()
    } else {
        true
    }
}

fn should_skip_serializing_labelling(maybe_labelling: &Option<Labelling>) -> bool {
    if let Some(labelling) = maybe_labelling {
        labelling.assigned.is_empty()
            && labelling.dismissed.is_empty()
            && should_skip_serializing_optional_vec(&labelling.predicted)
    } else {
        true
    }
}

fn should_skip_serializing_entities(maybe_entities: &Option<Entities>) -> bool {
    if let Some(entities) = maybe_entities {
        entities.assigned.is_empty()
            && entities.dismissed.is_empty()
            && should_skip_serializing_optional_vec(&entities.predicted)
    } else {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NewEntity {
    pub kind: EntityKind,
    pub formatted_value: String,
    pub span: NewEntitySpan,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct NewEntitySpan {
    content_part: String,
    message_index: usize,
    utf16_byte_start: usize,
    utf16_byte_end: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Entity {
    pub kind: EntityKind,
    pub formatted_value: String,
    pub span: EntitySpan,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct EntitySpan {
    content_part: String,
    message_index: usize,
    char_start: usize,
    char_end: usize,
    utf16_byte_start: usize,
    utf16_byte_end: usize,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct EntityKind(pub String);

impl FromStr for EntityKind {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Error> {
        Ok(EntityKind(string.to_owned()))
    }
}

#[inline]
fn strip_prefix(string: &mut String, prefix: &str) -> bool {
    if string.starts_with(prefix) {
        string.drain(..prefix.len());
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{self, json, Value as JsonValue};
    use std::collections::HashMap;

    #[test]
    fn property_map_empty_serialize() {
        assert_eq!(serde_json::to_string(&PropertyMap::new()).expect(""), "{}");
    }

    #[test]
    fn property_map_one_number_serialize() {
        let mut map = PropertyMap::new();
        map.insert_number("nps".to_owned(), 7.0);
        assert_eq!(
            serde_json::to_string(&map).expect(""),
            r#"{"number:nps":7.0}"#
        );
    }

    #[test]
    fn property_map_one_string_serialize() {
        let mut map = PropertyMap::new();
        map.insert_string("age".to_owned(), "18-25".to_owned());
        assert_eq!(
            serde_json::to_string(&map).expect(""),
            r#"{"string:age":"18-25"}"#
        );
    }

    #[test]
    fn property_map_mixed_serialize() {
        let mut map = PropertyMap::new();
        map.insert_string("age".to_owned(), "18-25".to_owned());
        map.insert_string("income".to_owned(), "$6000".to_owned());
        map.insert_number("nps".to_owned(), 3.0);

        let actual_map: HashMap<String, JsonValue> =
            serde_json::from_str(&serde_json::to_string(&map).expect("ser")).expect("de");

        let mut expected_map = HashMap::new();
        expected_map.insert(
            "string:age".to_owned(),
            JsonValue::String("18-25".to_owned()),
        );
        expected_map.insert(
            "string:income".to_owned(),
            JsonValue::String("$6000".to_owned()),
        );
        expected_map.insert("number:nps".to_owned(), json!(3.0));

        assert_eq!(actual_map, expected_map);
    }

    #[test]
    fn property_map_empty_deserialize() {
        let map: PropertyMap = serde_json::from_str("{}").expect("");
        assert_eq!(map, PropertyMap::new());
    }

    #[test]
    fn property_map_null_deserialize() {
        let map: PropertyMap = serde_json::from_str("null").expect("");
        assert_eq!(map, PropertyMap::new());
    }

    #[test]
    fn property_map_one_number_float_deserialize() {
        let actual: PropertyMap = serde_json::from_str(r#"{"number:nps":7.0}"#).expect("");
        let mut expected = PropertyMap::new();
        expected.insert_number("nps".to_owned(), 7.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn property_map_one_number_unsigned_deserialize() {
        let actual: PropertyMap = serde_json::from_str(r#"{"number:nps":7}"#).expect("");
        let mut expected = PropertyMap::new();
        expected.insert_number("nps".to_owned(), 7.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn property_map_one_number_negative_deserialize() {
        let actual: PropertyMap = serde_json::from_str(r#"{"number:nps":-7}"#).expect("");
        let mut expected = PropertyMap::new();
        expected.insert_number("nps".to_owned(), -7.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn property_map_one_string_deserialize() {
        let actual: PropertyMap = serde_json::from_str(r#"{"string:age":"18-25"}"#).expect("");
        let mut expected = PropertyMap::new();
        expected.insert_string("age".to_owned(), "18-25".to_owned());
        assert_eq!(actual, expected);
    }

    #[test]
    fn property_map_illegal_prefix_deserialize() {
        let result: Result<PropertyMap, _> =
            serde_json::from_str(r#"{"illegal:something":"18-25"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn property_map_illegal_value_for_prefix_deserialize() {
        let result: Result<PropertyMap, _> = serde_json::from_str(r#"{"string:something":18.0}"#);
        assert!(result.is_err());

        let result: Result<PropertyMap, _> = serde_json::from_str(r#"{"number:something":"x"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn property_map_mixed_deserialize() {
        let mut expected = PropertyMap::new();
        expected.insert_string("age".to_owned(), "18-25".to_owned());
        expected.insert_string("income".to_owned(), "$6000".to_owned());
        expected.insert_number("nps".to_owned(), 3.0);

        let actual: PropertyMap = serde_json::from_str(
            r#"{"string:age":"18-25","number:nps":3,"string:income":"$6000"}"#,
        )
        .expect("");

        assert_eq!(actual, expected);
    }
}
