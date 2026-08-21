use thiserror::Error;

#[derive(Debug, Error)]
pub enum IcaoError {
    #[error("invalid TLV tag: 0x{0:02X}")]
    InvalidTag(u8),

    #[error("invalid data length")]
    InvalidLength,

    #[error("invalid MRZ: {0}")]
    InvalidMrz(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("signature verification failed")]
    SignatureVerificationFailed,

    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),
}
