//! Error types for cluster analysis.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClusterError {
    #[error("cluster not found: {0}")]
    NotFound(uuid::Uuid),

    #[error("cluster already resolved")]
    AlreadyResolved,

    #[error("empty graph: no edges to cluster")]
    EmptyGraph,

    #[error("database: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),
}
