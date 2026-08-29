//! abis-attest crate.
#![warn(missing_docs)]
pub mod attestation;
pub mod error;
pub mod nonce;
pub mod policy;
pub mod sev;
pub mod tdx;
pub mod verify;

pub use attestation::{
    attest_batch, verify_attestation, AttestationProfile, AttestationResult, EnclaveAttestation,
};
pub use error::AttestError;
pub use nonce::{Nonce, NonceStore};
pub use policy::{AttestationPolicy, AttestationVerdict, ReportPayload};
pub use sev::{DevVcekStore, SevSnpReport, TrustedVcekStore};
pub use tdx::{DevPckStore, TdReport, TrustedPckStore};
pub use verify::AttestationVerifier;
