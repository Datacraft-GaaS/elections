//! Cryptographic primitives for the eel platform.
//!
//! This crate implements the cryptographic layer described in ADR-0005:
//!
//! - [`ed25519`]: Ed25519 sign / verify (pure Rust, always available).
//! - [`blake3`]: BLAKE3 hashing (pure Rust, always available).
//! - [`rfc8785`]: RFC 8785 JSON Canonicalization Scheme (pure Rust).
//! - [`merkle`]: RFC 6962 Merkle tree primitives (pure Rust).
//! - [`constant_time`]: Constant-time byte comparison wrapper.
//! - [`pkcs11_pool`]: PKCS#11 session pool (`pkcs11` feature, requires liboqs).
//! - [`pq_hybrid`]: ML-DSA-65 + ECDSA P-384 hybrid (`pq-hybrid` feature).
//! - [`pq_kem`]: ML-KEM-768 + X25519 key encapsulation (`pq-kem` feature).
//! - [`pq_slh`]: SLH-DSA archive signing (`pq-slh` feature).
//! - [`cosign`]: cosign bundle verification (`cosign` feature).
//! - [`tsa`]: RFC 3161 TSA client (`tsa` feature).
//!
//! # Key roles (ADR-0005)
//!
//! The [`KeyRole`] enum names every HSM-managed key purpose.

pub mod blake3;
pub mod constant_time;
pub mod ed25519;
pub mod error;
pub mod merkle;
pub mod rfc8785;

#[cfg(feature = "pkcs11")]
pub mod pkcs11_pool;

#[cfg(any(feature = "pq-hybrid", feature = "pq-slh", feature = "pq-kem"))]
pub mod pq;

#[cfg(feature = "tsa")]
pub mod tsa;

#[cfg(feature = "cosign")]
pub mod cosign;

pub use error::CryptoError;

// ---------------------------------------------------------------------------
// KeyRole (ADR-0005)
// ---------------------------------------------------------------------------

/// HSM key role identifiers per ADR-0005.
///
/// Each variant corresponds to a distinct HSM key object.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::KeyRole;
/// assert_eq!(KeyRole::Sth.label(), "eel-sth");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KeyRole {
    /// Signed Tree Head key: signs transparency-log tree heads.
    Sth,
    /// Result declaration key: signs M-of-N election result declarations.
    Declaration,
    /// Kit provisioning CA.
    KitCa,
    /// Credential issuance CA.
    CredentialCa,
    /// CI/CD code-signing key.
    CodeSigning,
    /// Cycle Key-Encryption Key (wraps per-cycle DEKs).
    CycleKek,
}

impl KeyRole {
    /// Returns the HSM key label string for this role.
    pub fn label(self) -> &'static str {
        match self {
            KeyRole::Sth => "eel-sth",
            KeyRole::Declaration => "eel-declaration",
            KeyRole::KitCa => "eel-kit-ca",
            KeyRole::CredentialCa => "eel-credential-ca",
            KeyRole::CodeSigning => "eel-code-signing",
            KeyRole::CycleKek => "eel-cycle-kek",
        }
    }
}
