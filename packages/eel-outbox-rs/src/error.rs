use thiserror::Error;

#[derive(Debug, Error)]
pub enum OutboxError {
    #[error("payload exceeds 1 MiB limit ({size} bytes); upload blobs to MinIO first")]
    PayloadTooLarge { size: usize },

    #[error("MinIO blob upload failed: {0}")]
    BlobUpload(String),

    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("invalid envelope: {0}")]
    InvalidEnvelope(String),
}
