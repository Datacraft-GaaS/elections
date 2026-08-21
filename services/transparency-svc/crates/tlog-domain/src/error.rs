use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("leaf hash mismatch")]
    HashMismatch,
    #[error("invalid tree size: {0}")]
    InvalidTreeSize(u64),
    #[error("invalid proof")]
    InvalidProof,
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("crypto error: {0}")]
    Crypto(#[from] eel_crypto_rs::CryptoError),
    #[error("unknown domain: {0}")]
    UnknownDomain(String),
}
