use chrono::{DateTime, Utc};
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    resources::{
        entity_def::{EntityDef, GeneralFieldDef, NewEntityDef, NewGeneralFieldDef},
        label_def::{LabelDef, NewLabelDef},
        label_group::{LabelGroup, NewLabelGroup},
        source::Id as SourceId,
        user::Username,
    },
    AnnotatedComment, CommentFilter, Continuation,
};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Dataset {
    pub id: Id,
    pub name: Name,
    pub owner: Username,
    pub title: String,
    pub description: String,
    #[serde(rename = "created")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "last_modified")]
    pub updated_at: DateTime<Utc>,
    pub model_family: ModelFamily,
    pub source_ids: Vec<SourceId>,
    pub has_sentiment: bool,
    pub entity_defs: Vec<EntityDef>,
    pub general_fields: Vec<GeneralFieldDef>,
    pub label_defs: Vec<LabelDef>,
    pub label_groups: Vec<LabelGroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DatasetStats {
    pub num_reviewed: NotNan<f64>,
    pub total_verbatims: NotNan<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DatasetAndStats {
    pub dataset: Dataset,
    pub stats: DatasetStats,
}

impl Dataset {
    pub fn full_name(&self) -> FullName {
        FullName(format!("{}/{}", self.owner.0, self.name.0))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FullName(pub String);

impl FromStr for FullName {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.split('/').count() == 2 {
            Ok(FullName(string.into()))
        } else {
            Err(Error::BadDatasetIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeResolution {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Attribute {
    Labels,
    AttachmentPropertyTypes,
    AttachmentPropertyNumAttachments,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttributeFilterEnum {
    StringAnyOf {
        any_of: Vec<String>,
    },
    NumberRange {
        minimum: Option<usize>,
        maximum: Option<usize>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct AttributeFilter {
    pub attribute: Attribute,
    pub filter: AttributeFilterEnum,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StatisticsRequestParams {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub attribute_filters: Vec<AttributeFilter>,

    pub comment_filter: CommentFilter,

    pub label_property_timeseries: bool,

    pub label_timeseries: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_resolution: Option<TimeResolution>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrderEnum {
    ByLabel { label: String },
    Recent,
    Sample { seed: usize },
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryRequestParams {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub attribute_filters: Vec<AttributeFilter>,

    pub filter: CommentFilter,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryRequestParams {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub attribute_filters: Vec<AttributeFilter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<Continuation>,

    pub filter: CommentFilter,

    pub limit: usize,

    pub order: OrderEnum,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPropertySummary {
    pub full_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPropertySummaryList {
    pub string: Vec<UserPropertySummary>,
    pub number: Vec<UserPropertySummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Summary {
    pub user_properties: UserPropertySummaryList,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SummaryResponse {
    pub summary: Summary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryResponse {
    pub continuation: Option<Continuation>,
    pub results: Vec<AnnotatedComment>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ModelFamily(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ModelVersion(pub u32);

// TODO(mcobzarenco)[3963]: Make `Identifier` into a trait (ensure it still implements
// `FromStr` so we can take T: Identifier as a clap command line argument).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Identifier {
    Id(Id),
    FullName(FullName),
}

impl From<FullName> for Identifier {
    fn from(full_name: FullName) -> Self {
        Identifier::FullName(full_name)
    }
}

impl From<Id> for Identifier {
    fn from(id: Id) -> Self {
        Identifier::Id(id)
    }
}

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Identifier::Id(Id(string.into())))
        } else {
            FullName::from_str(string).map(Identifier::FullName)
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "{}",
            match self {
                Identifier::Id(id) => &id.0,
                Identifier::FullName(full_name) => &full_name.0,
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewDataset<'request> {
    pub source_ids: &'request [SourceId],

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_sentiment: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_defs: Option<&'request [NewEntityDef]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub general_fields: Option<&'request [NewGeneralFieldDef]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_defs: Option<&'request [NewLabelDef]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_groups: Option<&'request [NewLabelGroup]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_family: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_annotations_from: Option<&'request str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CreateRequest<'request> {
    pub dataset: NewDataset<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CreateResponse {
    pub dataset: Dataset,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetAvailableResponse {
    pub datasets: Vec<Dataset>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetResponse {
    pub dataset: Dataset,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UpdateDataset<'request> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ids: Option<&'request [SourceId]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'request str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'request str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct UpdateRequest<'request> {
    pub dataset: UpdateDataset<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct UpdateResponse {
    pub dataset: Dataset,
}

#[cfg(test)]
mod tests {
    use crate::resources::comment::{CommentTimestampFilter, PropertyFilter, UserPropertiesFilter};

    use super::*;
    use crate::PropertyValue;
    use chrono::TimeZone;
    use std::collections::HashMap;

    #[test]
    pub fn test_serialize_query_params_recent() {
        let params = QueryRequestParams {
            filter: CommentFilter {
                user_properties: None,
                timestamp: Some(CommentTimestampFilter {
                    maximum: Some(
                        chrono::Utc
                            .with_ymd_and_hms(2023, 5, 19, 23, 59, 59)
                            .unwrap(),
                    ),
                    minimum: None,
                }),
                reviewed: Some(crate::resources::comment::ReviewedFilterEnum::OnlyUnreviewed),
                ..Default::default()
            },
            attribute_filters: vec![AttributeFilter {
                attribute: Attribute::Labels,
                filter: AttributeFilterEnum::StringAnyOf {
                    any_of: vec!["Access Management".to_string()],
                },
            }],
            continuation: Some(Continuation(
                "36498883b7f4c2c12cc364be0a44d806-8abb3088feffef3f".to_string(),
            )),
            limit: 20,
            order: OrderEnum::Recent,
        };

        assert_eq!(
            serde_json::to_string(&params).unwrap(),
            r#"{"attribute_filters":[{"attribute":"labels","filter":{"kind":"string_any_of","any_of":["Access Management"]}}],"continuation":"36498883b7f4c2c12cc364be0a44d806-8abb3088feffef3f","filter":{"reviewed":"only_unreviewed","timestamp":{"maximum":"2023-05-19T23:59:59Z"}},"limit":20,"order":{"kind":"recent"}}"#
        );
    }

    #[test]
    pub fn test_serialize_query_params_by_label() {
        let params = QueryRequestParams {
            filter: CommentFilter {
                user_properties: None,
                timestamp: Some(CommentTimestampFilter {
                    maximum: Some(
                        chrono::Utc
                            .with_ymd_and_hms(2023, 5, 19, 23, 59, 59)
                            .unwrap(),
                    ),
                    minimum: None,
                }),
                reviewed: Some(crate::resources::comment::ReviewedFilterEnum::OnlyUnreviewed),
                ..Default::default()
            },
            attribute_filters: vec![AttributeFilter {
                attribute: Attribute::Labels,
                filter: AttributeFilterEnum::StringAnyOf {
                    any_of: vec!["Access Management".to_string()],
                },
            }],
            continuation: Some(Continuation(
                "36498883b7f4c2c12cc364be0a44d806-8abb3088feffef3f".to_string(),
            )),
            limit: 20,
            order: OrderEnum::ByLabel {
                label: "Access Management".to_string(),
            },
        };

        assert_eq!(
            serde_json::to_string(&params).unwrap(),
            r#"{"attribute_filters":[{"attribute":"labels","filter":{"kind":"string_any_of","any_of":["Access Management"]}}],"continuation":"36498883b7f4c2c12cc364be0a44d806-8abb3088feffef3f","filter":{"reviewed":"only_unreviewed","timestamp":{"maximum":"2023-05-19T23:59:59Z"}},"limit":20,"order":{"kind":"by_label","label":"Access Management"}}"#
        );
    }

    #[test]
    pub fn test_serialize_statistics_request_params_default() {
        let params = StatisticsRequestParams::default();
        assert_eq!(
            serde_json::to_string(&params).unwrap(),
            "{\"comment_filter\":{},\"label_property_timeseries\":false,\"label_timeseries\":false}"
        )
    }

    #[test]
    pub fn test_serialize_statistics_request_params() {
        let params = StatisticsRequestParams {
            attribute_filters: vec![AttributeFilter {
                attribute: Attribute::Labels,
                filter: AttributeFilterEnum::StringAnyOf {
                    any_of: vec!["label Name".to_string()],
                },
            }],
            label_property_timeseries: true,
            label_timeseries: true,
            time_resolution: Some(TimeResolution::Day),
            comment_filter: CommentFilter {
                user_properties: None,
                reviewed: None,
                timestamp: Some(CommentTimestampFilter {
                    minimum: Some(
                        chrono::Utc
                            .with_ymd_and_hms(2019, 3, 17, 16, 43, 0)
                            .unwrap(),
                    ),
                    maximum: Some(
                        chrono::Utc
                            .with_ymd_and_hms(2020, 3, 17, 13, 33, 15)
                            .unwrap(),
                    ),
                }),
                ..Default::default()
            },
        };

        assert_eq!(
            serde_json::to_string(&params).unwrap(),
            r#"{"attribute_filters":[{"attribute":"labels","filter":{"kind":"string_any_of","any_of":["label Name"]}}],"comment_filter":{"timestamp":{"minimum":"2019-03-17T16:43:00Z","maximum":"2020-03-17T13:33:15Z"}},"label_property_timeseries":true,"label_timeseries":true,"time_resolution":"day"}"#
        )
    }

    #[test]
    pub fn test_serialize_user_properties_request_params() {
        let user_property_filter = UserPropertiesFilter(HashMap::from([(
            "string:Generation Tag".to_string(),
            PropertyFilter::new(
                vec![PropertyValue::String(
                    "72b01fe7-ef2e-481e-934d-bc2fe0ca9b06".to_string(),
                )],
                Vec::new(),
                Vec::new(),
            ),
        )]));

        let params = QueryRequestParams {
            attribute_filters: Vec::new(),
            continuation: None,
            limit: 20,
            order: OrderEnum::Recent,
            filter: CommentFilter {
                reviewed: None,
                timestamp: None,
                user_properties: Some(user_property_filter),
                ..Default::default()
            },
        };

        assert_eq!(
            serde_json::to_string(&params).unwrap(),
            r#"{"filter":{"user_properties":{"string:Generation Tag":{"one_of":["72b01fe7-ef2e-481e-934d-bc2fe0ca9b06"]}}},"limit":20,"order":{"kind":"recent"}}"#
        );
    }
}
