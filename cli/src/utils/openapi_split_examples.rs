//! Usage examples for the OpenAPI split-on-failure utility
//! 
//! This module shows how to use the `execute_with_split_on_failure` function
//! to add resilience to OpenAPI batch operations.

use anyhow::{Context, Result};
use openapi::{
    apis::{
        configuration::Configuration,
        comments_api::{add_comments, sync_comments, AddCommentsError, SyncCommentsError},
        emails_api::{add_emails_to_bucket, AddEmailsToBucketError},
    },
    models::{AddCommentsRequest, SyncCommentsRequest, AddEmailsToBucketRequest, CommentNew},
};
use crate::utils::openapi_split_on_failure::{execute_with_split_on_failure, SplitOnFailureResult};

/// Example: Add comments with split-on-failure
/// 
/// This shows how to wrap the `add_comments` OpenAPI call to provide automatic
/// splitting when batch requests fail.
pub fn add_comments_with_split_on_failure(
    config: &Configuration,
    owner: &str,
    source_name: &str,
    comments: Vec<CommentNew>,
) -> Result<SplitOnFailureResult<openapi::models::AddCommentsResponse>> {
    let request = AddCommentsRequest::new(comments);
    
    // Create a closure that captures the API parameters
    let api_call = |req: AddCommentsRequest| {
        add_comments(config, owner, source_name, req)
    };
    
    execute_with_split_on_failure(
        api_call,
        request,
        "add_comments"
    ).context("Failed to add comments with split-on-failure")
}

/// Example: Sync comments with split-on-failure  
/// 
/// Similar to add_comments but for sync operations which return more detailed
/// statistics that get properly merged across split requests.
pub fn sync_comments_with_split_on_failure(
    config: &Configuration,
    owner: &str,
    source_name: &str,
    comments: Vec<CommentNew>,
) -> Result<SplitOnFailureResult<openapi::models::SyncCommentsResponse>> {
    let request = SyncCommentsRequest::new(comments);
    
    let api_call = |req: SyncCommentsRequest| {
        sync_comments(config, owner, source_name, req)
    };
    
    execute_with_split_on_failure(
        api_call,
        request,
        "sync_comments"
    ).context("Failed to sync comments with split-on-failure")
}

/// Example: Add emails to bucket with split-on-failure
pub fn add_emails_to_bucket_with_split_on_failure(
    config: &Configuration,
    owner: &str,
    bucket_name: &str,
    emails: Vec<openapi::models::EmailNew>,
) -> Result<SplitOnFailureResult<openapi::models::AddEmailsToBucketResponse>> {
    let request = AddEmailsToBucketRequest::new(emails);
    
    let api_call = |req: AddEmailsToBucketRequest| {
        add_emails_to_bucket(config, owner, bucket_name, req)
    };
    
    execute_with_split_on_failure(
        api_call,
        request,
        "add_emails_to_bucket"
    ).context("Failed to add emails to bucket with split-on-failure")
}

/// Example: Integration into existing code
/// 
/// Shows how you can easily replace existing OpenAPI calls with split-on-failure versions.
/// 
/// ```rust,no_run
/// # use anyhow::Result;
/// # use openapi::{apis::configuration::Configuration, models::CommentNew};
/// # fn example(config: &Configuration, owner: &str, source: &str, comments: Vec<CommentNew>) -> Result<()> {
/// // Before: Direct OpenAPI call (fails completely on any error)
/// // let response = add_comments(config, owner, source, AddCommentsRequest::new(comments))?;
/// 
/// // After: With split-on-failure (partial success possible)
/// let result = add_comments_with_split_on_failure(config, owner, source, comments)?;
/// 
/// println!("Successfully processed {} comments", 
///          comments.len() - result.num_failed);
/// if result.num_failed > 0 {
///     println!("Failed to process {} comments", result.num_failed);
/// }
/// # Ok(())
/// # }
/// ```
pub fn integration_example() {
    // This function exists only for documentation purposes
    // The actual example is in the docstring above
}

/// Helper function to demonstrate error handling patterns
/// 
/// Shows how to handle the results of split-on-failure operations,
/// including partial failures.
pub fn handle_split_on_failure_result<T>(
    result: SplitOnFailureResult<T>,
    total_items: usize,
    operation_name: &str,
) {
    let successful = total_items - result.num_failed;
    
    if result.num_failed == 0 {
        println!("✅ {} completed successfully: {}/{} items processed", 
                operation_name, successful, total_items);
    } else if successful > 0 {
        println!("⚠️  {} partially completed: {}/{} items processed, {} failed", 
                operation_name, successful, total_items, result.num_failed);
    } else {
        println!("❌ {} failed completely: 0/{} items processed", 
                operation_name, total_items);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_result_display() {
        // Test complete success
        let success_result = SplitOnFailureResult {
            response: openapi::models::AddCommentsResponse::default(),
            num_failed: 0,
        };
        handle_split_on_failure_result(success_result, 10, "test_operation");
        
        // Test partial success
        let partial_result = SplitOnFailureResult {
            response: openapi::models::AddCommentsResponse::default(),
            num_failed: 3,
        };
        handle_split_on_failure_result(partial_result, 10, "test_operation");
        
        // Test complete failure
        let failure_result = SplitOnFailureResult {
            response: openapi::models::AddCommentsResponse::default(),
            num_failed: 10,
        };
        handle_split_on_failure_result(failure_result, 10, "test_operation");
    }
}

