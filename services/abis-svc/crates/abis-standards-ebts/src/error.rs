use thiserror::Error;

/// Errors produced by FBI EBTS parsing and serialisation.
#[derive(Debug, Error)]
pub enum EbtsError {
    #[error("unknown TOT code: {0}")]
    UnknownTot(String),

    #[error("invalid format: {0}")]
    InvalidFormat(String),

    #[error("missing required field: {0}")]
    MissingField(String),
}
