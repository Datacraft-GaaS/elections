//! Error types for the zk-tally crate.

/// Errors produced by zk-tally operations.
#[derive(Debug, thiserror::Error)]
pub enum ZkError {
    #[error("constraint synthesis failed: {0}")]
    ConstraintSynthesis(String),

    #[error("trusted setup failed: {0}")]
    Setup(String),

    #[error("proof generation failed: {0}")]
    Prove(String),

    #[error("proof verification failed: {0}")]
    Verify(String),

    #[error("serialisation error: {0}")]
    Serialise(String),

    #[error("leaf count {0} exceeds maximum {1}")]
    TooManyLeaves(usize, usize),

    #[error("tally mismatch: leaves sum to {leaves_sum}, published = {published}")]
    TallyMismatch { leaves_sum: u64, published: u64 },
}
