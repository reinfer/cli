use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GetResponse {
    pub statistics: Statistics,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Statistics {
    pub num_comments: usize,
}
