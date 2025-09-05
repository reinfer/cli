use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Comment timestamp filter, similar to the reinfer_client CommentTimestampFilter
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommentTimestampFilter {
    pub minimum: Option<DateTime<Utc>>,
    pub maximum: Option<DateTime<Utc>>,
}

impl CommentTimestampFilter {
    pub fn new(minimum: Option<DateTime<Utc>>, maximum: Option<DateTime<Utc>>) -> Self {
        Self { minimum, maximum }
    }
}
