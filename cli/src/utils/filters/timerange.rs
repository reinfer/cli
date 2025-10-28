use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Timerange for comments iteration, similar to the reinfer_client CommentsIterTimerange
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommentsIterTimerange {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl CommentsIterTimerange {
    pub fn new(from: Option<DateTime<Utc>>, to: Option<DateTime<Utc>>) -> Self {
        Self { from, to }
    }
}
