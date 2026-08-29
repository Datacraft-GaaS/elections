//! abis-modality-ear crate.
#![warn(missing_docs)]
pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use domain::{EarTemplate, Point2D, NUM_LANDMARKS};
pub use matcher::{match_ear, Score, DEFAULT_THRESHOLD};
pub use parser::{parse_ear, serialize_ear};
pub use quality::{score_quality, EarQuality};

#[derive(Debug, thiserror::Error)]
pub enum EarError {
    #[error("invalid magic bytes: expected EARL")]
    InvalidMagic,
    #[error("buffer too short: need {need} bytes, got {got}")]
    BufferTooShort { need: usize, got: usize },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
