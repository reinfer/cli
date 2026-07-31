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
    AnnotatedComment, CommentFilter, CommentId, Continuation, ProjectName,
};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use super::validation::ValidationResponse;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetFlag {
    Gpt4,
    ExternalMoonLlm,
    Qos,
    ZeroShotLabels,
    Ixp,
    ConversationalFilters,
    GenerativeExtraction,
    GenerativePrelabelling,
    LlmAssistedLabelling,
    /// A dataset flag added to the platform after this release, kept as-is.
    #[serde(untagged)]
    Unknown(Box<str>),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct IxpDatasetNew {
    pub name: Name,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateIxpDatasetRequest {
    pub dataset: IxpDatasetNew,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct UploadIxpDocumentResponse {
    pub comment_id: CommentId,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateIxpDatasetResponse {
    pub dataset: Dataset,
}

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
    #[serde(rename = "_dataset_flags")]
    pub dataset_flags: Vec<DatasetFlag>,
    #[serde(rename = "_model_config", default)]
    pub model_config: ModelConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[derive(Default)]
pub enum ModelConfig {
    #[default]
    Cm,
    DocPathIxp,
    GptIxp(GptIxpModelConfig),
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocPathIxpModelConfig {
    #[serde(
        rename = "num_pages_per_chunk",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub num_pages_per_chunk: Option<Option<i32>>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct GptIxpModelConfig {
    #[serde(
        rename = "num_pages_per_chunk",
        skip_serializing_if = "Option::is_none"
    )]
    pub num_pages_per_chunk: Option<i32>,
    #[serde(rename = "model_version", skip_serializing_if = "Option::is_none")]
    pub model_version: Option<GptModelVersion>,
    #[serde(
        rename = "input_config",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub input_config: Option<Option<IxpInputConfig>>,
    #[serde(
        rename = "system_prompt_override",
        skip_serializing_if = "Option::is_none"
    )]
    pub system_prompt_override: Option<String>,
    #[serde(rename = "frequency_penalty", skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<NotNan<f64>>,
    #[serde(rename = "temperature", skip_serializing_if = "Option::is_none")]
    pub temperature: Option<NotNan<f64>>,
    #[serde(rename = "top_p", skip_serializing_if = "Option::is_none")]
    pub top_p: Option<NotNan<f64>>,
    #[serde(rename = "seed", skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(rename = "flags")]
    pub flags: Vec<GptIxpFlag>,
    #[serde(rename = "iterative_config", skip_serializing_if = "Option::is_none")]
    pub iterative_config: Option<IterativeConfig>,

    #[serde(default)]
    pub attribution_method: AttributionMethod,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AttributionMethod {
    #[default]
    TableHeuristic,
    WordIds,
    TableFormattedWordIds,
    /// An attribution method added to the platform after this release, kept as-is.
    #[serde(untagged)]
    Unknown(Box<str>),
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct IterativeConfig {
    #[serde(
        rename = "max_num_calls",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_num_calls: Option<Option<i32>>,
    #[serde(
        rename = "chunk_size",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub chunk_size: Option<Option<NotNan<f64>>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum GptModelVersion {
    #[serde(rename = "gpt_4o_2024_05_13")]
    Gpt4o20240513,
    #[serde(rename = "gpt_5_1_2025_11_13")]
    Gpt5120251113,
    #[serde(rename = "gpt_5_4_2026_03_05")]
    Gpt5420260305,
    #[serde(rename = "gemini_2_5_flash")]
    GeminiFlash25,
    #[serde(rename = "gemini_2_5_pro")]
    GeminiPro25,
    #[serde(rename = "gemini_3_1_pro_preview")]
    Gemini31ProPreview,
    #[serde(rename = "gemini_3_1_flash_lite_preview")]
    Gemini31FlashLitePreview,
    /// A model version added to the platform after this release, kept as-is.
    #[serde(untagged)]
    Unknown(Box<str>),
}

#[derive(Eq, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IxpInputConfig {
    TextImageInputConfig(TextImageInputConfig),
    ImageInputConfig(ImageInputConfig),
}

#[derive(Eq, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextImageInputConfig {
    #[serde(rename = "mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<TextImageInputConfigMode>,
    #[serde(rename = "text_config")]
    pub text_config: TextConfig,
}

#[derive(Eq, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageInputConfig {
    #[serde(rename = "mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<ImageInputConfigMode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ImageInputConfigMode {
    #[serde(rename = "image_only")]
    ImageOnly,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct TextConfig {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TextImageInputConfigMode {
    #[serde(rename = "text_plus_image")]
    TextPlusImage,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GptIxpFlag {
    AppendTaxonomyDescriptions,
    AppendTypeDescriptions,
    AppendGroupDescriptions,
    AppendFieldDescriptions,
    /// An extraction flag added to the platform after this release, kept as-is.
    #[serde(untagged)]
    Unknown(Box<str>),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DatasetStats {
    pub total_verbatims: NotNan<f64>,
    pub validation: Option<ValidationResponse>,
    pub number_of_labels: usize,
    pub number_of_fields: usize,
    pub number_of_extraction_defs: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DatasetAndStats {
    pub dataset: Dataset,
    pub stats: DatasetStats,
}

impl Dataset {
    pub fn full_name(&self) -> FullName {
        FullName(format!("{}/{}", self.owner.0, self.name.0))
    }

    pub fn has_flag(&self, flag: DatasetFlag) -> bool {
        self.dataset_flags.contains(&flag)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

impl Name {
    pub fn with_project(self, project: &ProjectName) -> Result<FullName> {
        FullName::from_str(&format!("{0}/{1}", project.0, self.0))
    }
}
impl FromStr for Name {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Name(s.to_string()))
    }
}

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

impl Display for FullName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
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
pub struct GetAllModelsInDatasetRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModelMetadata {
    pub version: ModelVersion,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GetAllModelsInDatasetRespone {
    pub labellers: Vec<UserModelMetadata>,
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
#[derive(Default)]
pub enum OrderEnum {
    ByLabel {
        label: String,
    },
    #[default]
    Recent,
    Sample {
        seed: usize,
    },
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SummaryRequestParams {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub attribute_filters: Vec<AttributeFilter>,

    pub filter: CommentFilter,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct QueryRequestParams {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub attribute_filters: Vec<AttributeFilter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<Continuation>,

    pub filter: CommentFilter,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    pub order: OrderEnum,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPropertySummaryValue {
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPropertySummaryString {
    pub full_name: String,
    pub values: Vec<UserPropertySummaryValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPropertySummaryNumber {
    pub full_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPropertySummaryList {
    pub string: Vec<UserPropertySummaryString>,
    pub number: Vec<UserPropertySummaryNumber>,
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

impl std::fmt::Display for ModelVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "_dataset_flags")]
    pub dataset_flags: Vec<DatasetFlag>,
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

    #[serde(rename = "_model_config", skip_serializing_if = "Option::is_none")]
    pub model_config: Option<ModelConfig>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entity_defs: Vec<NewEntityDef>,
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
            limit: Some(20),
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
            limit: Some(20),
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
            limit: Some(20),
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

    // Each table lists every value the platform currently sends, so that known
    // values get a real variant instead of falling into `Unknown`.

    /// Asserts each value deserializes to the expected variant and serializes
    /// back unchanged.
    fn assert_round_trips<T>(cases: &[(&str, T)])
    where
        T: serde::de::DeserializeOwned + Serialize + PartialEq + std::fmt::Debug,
    {
        for (wire, expected) in cases {
            let json = format!("\"{wire}\"");
            let parsed: T = serde_json::from_str(&json)
                .unwrap_or_else(|error| panic!("`{wire}` should deserialize: {error}"));
            assert_eq!(
                &parsed, expected,
                "`{wire}` deserialized to the wrong variant"
            );
            assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
        }
    }

    #[test]
    fn test_every_dataset_flag_round_trips() {
        assert_round_trips(&[
            ("gpt4", DatasetFlag::Gpt4),
            ("external_moon_llm", DatasetFlag::ExternalMoonLlm),
            ("qos", DatasetFlag::Qos),
            ("zero_shot_labels", DatasetFlag::ZeroShotLabels),
            ("ixp", DatasetFlag::Ixp),
            ("conversational_filters", DatasetFlag::ConversationalFilters),
            ("generative_extraction", DatasetFlag::GenerativeExtraction),
            (
                "generative_prelabelling",
                DatasetFlag::GenerativePrelabelling,
            ),
            ("llm_assisted_labelling", DatasetFlag::LlmAssistedLabelling),
            (
                "some_future_flag",
                DatasetFlag::Unknown("some_future_flag".into()),
            ),
        ]);
    }

    #[test]
    fn test_every_attribution_method_round_trips() {
        assert_round_trips(&[
            ("table_heuristic", AttributionMethod::TableHeuristic),
            ("word_ids", AttributionMethod::WordIds),
            (
                "table_formatted_word_ids",
                AttributionMethod::TableFormattedWordIds,
            ),
            (
                "some_future_attribution",
                AttributionMethod::Unknown("some_future_attribution".into()),
            ),
        ]);
    }

    #[test]
    fn test_every_model_version_round_trips() {
        assert_round_trips(&[
            ("gpt_4o_2024_05_13", GptModelVersion::Gpt4o20240513),
            ("gpt_5_1_2025_11_13", GptModelVersion::Gpt5120251113),
            ("gpt_5_4_2026_03_05", GptModelVersion::Gpt5420260305),
            ("gemini_2_5_flash", GptModelVersion::GeminiFlash25),
            ("gemini_2_5_pro", GptModelVersion::GeminiPro25),
            (
                "gemini_3_1_pro_preview",
                GptModelVersion::Gemini31ProPreview,
            ),
            (
                "gemini_3_1_flash_lite_preview",
                GptModelVersion::Gemini31FlashLitePreview,
            ),
            ("gpt_9", GptModelVersion::Unknown("gpt_9".into())),
        ]);
    }

    #[test]
    fn test_every_gpt_ixp_flag_round_trips() {
        assert_round_trips(&[
            (
                "append_taxonomy_descriptions",
                GptIxpFlag::AppendTaxonomyDescriptions,
            ),
            (
                "append_type_descriptions",
                GptIxpFlag::AppendTypeDescriptions,
            ),
            (
                "append_group_descriptions",
                GptIxpFlag::AppendGroupDescriptions,
            ),
            (
                "append_field_descriptions",
                GptIxpFlag::AppendFieldDescriptions,
            ),
            (
                "append_something_new",
                GptIxpFlag::Unknown("append_something_new".into()),
            ),
        ]);
    }

    /// Listing commands parse every dataset on the tenant at once, so one
    /// dataset using newer values must not fail the whole response.
    #[test]
    fn test_deserialize_tenant_listing_with_newer_model_config_values() {
        let response: GetAvailableResponse = serde_json::from_str(
            r#"{"datasets":[
              {"id":"aaaaaaaaaaaaaaaa","name":"unrelated","owner":"proj","title":"Unrelated",
               "description":"","created":"2026-01-01T00:00:00Z",
               "last_modified":"2026-01-01T00:00:00Z","model_family":"english","source_ids":[],
               "has_sentiment":false,"entity_defs":[],"general_fields":[],"label_defs":[],
               "label_groups":[],
               "_dataset_flags":["generative_extraction","a_flag_from_the_future"],
               "_model_config":{"kind":"cm"}},
              {"id":"bbbbbbbbbbbbbbbb","name":"ixp-one","owner":"proj","title":"IXP",
               "description":"","created":"2026-01-01T00:00:00Z",
               "last_modified":"2026-01-01T00:00:00Z","model_family":"english","source_ids":[],
               "has_sentiment":false,"entity_defs":[],"general_fields":[],"label_defs":[],
               "label_groups":[],"_dataset_flags":["ixp"],
               "_model_config":{"kind":"gpt_ixp","flags":["a_flag_from_the_future"],
                                "model_version":"a_model_from_the_future",
                                "attribution_method":"table_formatted_word_ids"}}
            ]}"#,
        )
        .expect("a tenant listing must parse even when a dataset uses newer values");

        assert_eq!(response.datasets.len(), 2);
        assert!(response.datasets[0].has_flag(DatasetFlag::GenerativeExtraction));
        assert!(
            response.datasets[0].has_flag(DatasetFlag::Unknown("a_flag_from_the_future".into()))
        );

        let ModelConfig::GptIxp(config) = &response.datasets[1].model_config else {
            panic!("expected a gpt_ixp config");
        };
        assert_eq!(
            config.attribution_method,
            AttributionMethod::TableFormattedWordIds
        );
        assert_eq!(
            config.model_version,
            Some(GptModelVersion::Unknown("a_model_from_the_future".into()))
        );

        // `re package upload` sends the config straight back, so unrecognised
        // values also have to re-serialize exactly as they arrived.
        assert_eq!(
            serde_json::to_string(&response.datasets[1].model_config).unwrap(),
            r#"{"kind":"gpt_ixp","model_version":"a_model_from_the_future","flags":["a_flag_from_the_future"],"attribution_method":"table_formatted_word_ids"}"#
        );
    }
}
