use thiserror::Error;

/// Errors produced by CBEFF container parsing and serialisation.
#[derive(Debug, Error)]
pub enum CbeffError {
    #[error("invalid magic bytes (expected CBEFF header)")]
    InvalidMagic,

    #[error("invalid or truncated length field")]
    InvalidLength,

    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("nested CBEFF error: {0}")]
    NestedError(#[from] Box<CbeffError>),
}
