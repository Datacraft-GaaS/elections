pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use domain::{GaitCycle, GaitFrame, GaitTemplate, JointAngle, NUM_JOINTS};
pub use matcher::{match_gait, Score, DEFAULT_THRESHOLD};
pub use parser::{parse_gait, serialize_gait};
pub use quality::{score_quality, GaitQuality};

#[derive(Debug, thiserror::Error)]
pub enum GaitError {
    #[error("invalid magic bytes: expected GAIT")]
    InvalidMagic,
    #[error("buffer too short: need at least {need} bytes, got {got}")]
    BufferTooShort { need: usize, got: usize },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
