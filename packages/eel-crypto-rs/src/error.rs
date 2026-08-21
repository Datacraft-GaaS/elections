//! Unified error type for eel-crypto-rs.
//!
//! # Example
//!
//! ```
//! use eel_crypto_rs::CryptoError;
//! let e = CryptoError::InvalidSignature;
//! assert!(!e.to_string().is_empty());
//! ```

use thiserror::Error;

/// Errors produced by cryptographic operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CryptoError {
    /// The signature did not verify against the provided key and message.
    #[error("signature verification failed")]
    InvalidSignature,

    /// The supplied key material was malformed or the wrong size.
    #[error("invalid key: {0}")]
    InvalidKey(String),

    /// A Merkle proof was structurally invalid or did not verify.
    #[error("Merkle proof invalid: {0}")]
    InvalidMerkleProof(String),

    /// RFC 8785 canonicalisation failed (e.g. non-finite number).
    #[error("JSON canonicalisation failed: {0}")]
    CanonicalJson(String),

    /// PKCS#11 operation failed.
    #[cfg(feature = "pkcs11")]
    #[error("PKCS#11 error: {0}")]
    Pkcs11(#[from] cryptoki::error::Error),

    /// TSA HTTP or parsing error.
    #[cfg(feature = "tsa")]
    #[error("TSA error: {0}")]
    Tsa(String),

    /// cosign bundle verification error.
    #[cfg(feature = "cosign")]
    #[error("cosign verification failed: {0}")]
    Cosign(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Hex decoding error.
    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),

    /// Base-64 decoding error.
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic internal error.
    #[error("internal crypto error: {0}")]
    Internal(String),
}
