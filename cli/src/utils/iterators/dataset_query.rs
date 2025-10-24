use anyhow::{Context, Result};
use openapi::{
    apis::{comments_api::query_comments, configuration::Configuration},
    models::{AnnotatedComment, QueryCommentsRequest, QueryCommentsResponse},
};

/// Create a new dataset query iterator
pub fn get_dataset_query_iter(
    config: Configuration,
    dataset_owner: String,
    dataset_name: String,
    request: QueryCommentsRequest,
) -> DatasetQueryIter {
    DatasetQueryIter::new(config, dataset_owner, dataset_name, request)
}

/// Iterator for paginating through dataset query results using OpenAPI client
pub struct DatasetQueryIter {
    config: Configuration,
    dataset_owner: String,
    dataset_name: String,
    request: QueryCommentsRequest,
    current_batch: Option<Vec<AnnotatedComment>>,
    batch_index: usize,
    finished: bool,
}

impl DatasetQueryIter {
    pub fn new(
        config: Configuration,
        dataset_owner: String,
        dataset_name: String,
        mut request: QueryCommentsRequest,
    ) -> Self {
        // Ensure we start with no continuation
        request.continuation = None;

        Self {
            config,
            dataset_owner,
            dataset_name,
            request,
            current_batch: None,
            batch_index: 0,
            finished: false,
        }
    }

    /// Fetch the next batch of comments from the API
    fn fetch_next_batch(&mut self) -> Result<Option<QueryCommentsResponse>> {
        if self.finished {
            return Ok(None);
        }

        let response = query_comments(
            &self.config,
            &self.dataset_owner,
            &self.dataset_name,
            self.request.clone(),
            None, // limit is already in the request
            self.request.continuation.as_deref(),
        )
        .context("Failed to query comments")?;

        // Update continuation for next iteration
        self.request.continuation = response.continuation.clone();

        // Mark as finished if no continuation or empty results
        if response.continuation.is_none() || response.results.is_empty() {
            self.finished = true;
        }

        Ok(Some(response))
    }
}

impl Iterator for DatasetQueryIter {
    type Item = Result<Vec<AnnotatedComment>>;

    fn next(&mut self) -> Option<Self::Item> {
        // If we have a current batch and haven't finished iterating through it
        if let Some(batch) = &self.current_batch {
            if self.batch_index < batch.len() {
                // Still items in current batch - but this iterator returns whole pages
                // So we consume the whole batch at once
                let result = batch.clone();
                self.current_batch = None;
                self.batch_index = 0;
                return Some(Ok(result));
            }
        }

        // Need to fetch next batch
        match self.fetch_next_batch() {
            Ok(Some(response)) => {
                if response.results.is_empty() {
                    None
                } else {
                    self.current_batch = Some(response.results.clone());
                    self.batch_index = 0;
                    Some(Ok(response.results))
                }
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
