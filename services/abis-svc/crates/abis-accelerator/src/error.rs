//! Error types for accelerators.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AcceleratorError {
    #[error("accelerator not available: {0}")]
    NotAvailable(String),

    #[error("batch too large: max {max}, got {got}")]
    BatchTooLarge { max: usize, got: usize },

    #[error("probe/gallery template size mismatch")]
    TemplateSizeMismatch,

    #[error("accelerator unhealthy")]
    Unhealthy,

    #[error("runtime error: {0}")]
    Runtime(String),
}
