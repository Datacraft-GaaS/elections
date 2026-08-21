use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExplainError {
    #[error("template is empty")]
    EmptyTemplate,
    #[error("dimension mismatch")]
    DimensionMismatch,
    #[error("no match data available")]
    NoMatchData,
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
