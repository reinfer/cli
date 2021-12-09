use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub(crate) struct GetResponse {
    pub statistics: Statistics,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Statistics {
    pub num_comments: f64,
}
