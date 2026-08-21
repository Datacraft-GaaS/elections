use thiserror::Error;

/// Errors produced by ANSI/NIST-ITL parsing and serialisation.
#[derive(Debug, Error)]
pub enum NistError {
    #[error("invalid format: {0}")]
    InvalidFormat(String),

    #[error("unknown record type: {0}")]
    UnknownRecordType(u8),

    #[error("missing required field: {0}")]
    FieldMissing(u32),

    #[error("I/O error: {0}")]
    IoError(String),
}
