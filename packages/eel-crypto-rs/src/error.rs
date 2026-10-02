// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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

    /// An AEAD operation failed. Deliberately coarse: a wrong key, a modified
    /// ciphertext, a modified tag, and mismatched associated data are all
    /// reported identically so the error reveals nothing to an attacker
    /// probing the store.
    #[error("authenticated encryption or decryption failed")]
    Aead,

    /// The operating system random number generator failed.
    #[error("secure randomness unavailable")]
    Randomness,

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
