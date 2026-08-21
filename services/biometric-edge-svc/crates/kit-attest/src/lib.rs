//! Kit attestation verification (WP-3.8).
//!
//! Accepts Play Integrity, App Attest, TPM quote, or legacy software
//! attestation payloads and returns an AttestationVerdict.
pub mod verdict;
pub mod verify;

pub use verdict::{AttestationVerdict, IntegrityLevel, KitTier};
pub use verify::{verify_attestation, AttestationPayload, AttestationKind, AttestError};
