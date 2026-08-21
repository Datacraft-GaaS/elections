use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdhocError {
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("preview: {0}")]
    Preview(String),
    #[error("publish: {0}")]
    Publish(String),
    #[error("render: {0}")]
    Render(String),
    #[error("query too large: {rows} rows exceeds limit {limit}")]
    TooManyRows { rows: usize, limit: usize },
    #[error("statement timeout")]
    Timeout,
    #[error("memory cap exceeded: {bytes} bytes")]
    MemoryCap { bytes: usize },
}

pub type Result<T, E = AdhocError> = std::result::Result<T, E>;
