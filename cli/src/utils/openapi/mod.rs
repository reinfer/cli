pub mod api_ext;
pub mod split_examples;
pub mod split_failure;

// Re-export public items for convenience
pub use api_ext::{get_document_bytes, set_comment_audio, upload_ixp_document_bytes};
pub use split_examples::{
    add_comments_with_split_on_failure, add_emails_to_bucket_with_split_on_failure,
    handle_split_on_failure_result, sync_comments_with_split_on_failure,
};
// Note: execute_with_split_on_failure and SplitOnFailureResult are used internally by split_examples
// but not exposed at the top level since users should use the high-level wrapper functions
