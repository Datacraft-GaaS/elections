//! Reproducibility bundle assembler and verifier (WP-4.6, M14).
//!
//! Builds and verifies the court-defensible reproducibility bundle: a manifest
//! of every component (forms, tokens, source, build logs, SBOM) with SHA-256
//! digests and an optional Ed25519 signature over the canonical manifest. The
//! bundle is what lets an independent auditor rebuild the tally from committed
//! events and confirm the published Merkle root bit-for-bit.

pub mod bundle;
pub mod error;
pub mod manifest;
pub mod token;

pub use bundle::{Bundle, BundleComponent, ComponentType};
pub use error::ReproError;
pub use manifest::{BundleManifest, ManifestVersion};
pub use token::{InclusionProof, ParticipationToken};
