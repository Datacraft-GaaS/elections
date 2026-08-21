use thiserror::Error;

#[derive(Debug, Error)]
pub enum MinexError {
    #[error("invalid FMR header: {0}")]
    InvalidHeader(String),

    #[error("minutiae count {found} exceeds maximum {max}")]
    InvalidMinutiaeCount { found: usize, max: usize },

    #[error("coordinate ({x}, {y}) outside valid range")]
    InvalidCoordinate { x: u16, y: u16 },

    #[error("quality value {0} exceeds maximum 100")]
    InvalidQuality(u8),

    #[error("template checksum failed")]
    ChecksumFailed,
}
