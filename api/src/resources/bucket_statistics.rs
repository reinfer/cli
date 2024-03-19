use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GetBucketStatisticsResponse {
    pub statistics: Statistics,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Statistics {
    pub count: usize,
}
