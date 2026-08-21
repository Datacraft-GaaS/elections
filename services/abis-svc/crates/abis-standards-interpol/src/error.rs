use thiserror::Error;

#[derive(Debug, Error)]
pub enum InterpolError {
    #[error("invalid NCB code: {0}")]
    InvalidNcbCode(String),

    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid format: {0}")]
    InvalidFormat(String),
}
