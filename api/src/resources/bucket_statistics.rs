use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GetBucketStatisticsResponse {
    pub statistics: Statistics,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind")]
pub enum Count {
    #[serde(rename = "lower_bound")]
    LowerBoundBucketCount { value: i32 },
    #[serde(rename = "exact")]
    ExactBucketCount { value: i32 },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Statistics {
    pub count: Count,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LowerBoundBucketCount {
    pub kind: String,
    pub value: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExactBucketCount {
    pub kind: String,
    pub value: i32,
}
