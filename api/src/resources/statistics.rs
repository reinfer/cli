use std::collections::HashMap;

use crate::LabelName;
use chrono::NaiveDate;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub(crate) struct GetResponse {
    pub statistics: Statistics,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LabelCount {
    positive: f32,
    negative: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum TimeSeriesEntry {
    Timestamp(NaiveDate),
    Number(f32),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Statistics {
    pub num_comments: NotNan<f64>,

    #[serde(default)]
    pub label_counts: HashMap<LabelName, LabelCount>,

    #[serde(default)]
    pub label_timeseries: Vec<Vec<TimeSeriesEntry>>,
}
