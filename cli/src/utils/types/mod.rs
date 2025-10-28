pub mod identifiers;
pub mod ids;
pub mod model;
pub mod names;
pub mod project;
pub mod stream;
pub mod transform;

// Re-export public items for convenience
pub use identifiers::{
    resolve_bucket, resolve_dataset, resolve_source, BucketIdentifier, DatasetIdentifier,
    SourceIdentifier,
};
pub use ids::{CommentId, SourceId};
pub use model::ModelVersion;
pub use names::FullName;
pub use project::ProjectName;
pub use stream::StreamFullName;

// Type aliases for backward compatibility
pub type DatasetFullName = FullName;
