//! eel-abis-cert-fido: FIDO Biometric Component Certification harness.
//!
//! Generates certification evidence: PAD species coverage, PAI matrix,
//! FMR/FNMR report, and a tester signature stub.
#![warn(missing_docs)]

pub mod evidence;
pub mod matrix;

pub use evidence::{EvidenceError, EvidencePack};
pub use matrix::{PaiMatrix, PaiMatrixEntry, PaiMatrixError};
