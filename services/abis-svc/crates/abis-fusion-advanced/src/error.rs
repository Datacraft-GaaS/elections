use thiserror::Error;

#[derive(Debug, Error)]
pub enum FusionError {
    #[error("scores list is empty")]
    EmptyScores,
    #[error("dimension mismatch: expected {expected}, found {found}")]
    DimensionMismatch { expected: usize, found: usize },
    #[error("invalid score {0}: must be in [0,1]")]
    InvalidScore(f64),
    #[error("weight sum is zero")]
    WeightSumZero,
    #[error("insufficient cohort data: {0}")]
    InsufficientCohortData(String),
}
