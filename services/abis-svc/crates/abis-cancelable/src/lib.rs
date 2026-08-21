//! eel-abis-cancelable: ISO/IEC 24745 cancelable biometrics.
//!
//! Implements BioHashing, random projection (Johnson-Lindenstrauss), fuzzy
//! vault, fuzzy commitment, revocation, and renewal primitives.

pub mod biohash;
pub mod fuzzy_commit;
pub mod fuzzy_vault;
pub mod random_projection;
pub mod renewal;
pub mod revocation;

pub use biohash::{BioHash, BioHashError, BioHasher};
pub use fuzzy_commit::{FuzzyCommit, FuzzyCommitError};
pub use fuzzy_vault::{FuzzyVault, FuzzyVaultError};
pub use random_projection::{ProjectionError, RandomProjector};
pub use renewal::{RenewError, RenewalRecord};
pub use revocation::{RevocationError, RevocationRecord};

/// Common cancelable-biometrics error.
#[derive(Debug, thiserror::Error)]
pub enum CancelableError {
    #[error("biohash: {0}")]
    BioHash(#[from] BioHashError),
    #[error("fuzzy vault: {0}")]
    FuzzyVault(#[from] FuzzyVaultError),
    #[error("fuzzy commit: {0}")]
    FuzzyCommit(#[from] FuzzyCommitError),
    #[error("projection: {0}")]
    Projection(#[from] ProjectionError),
    #[error("revocation: {0}")]
    Revocation(#[from] RevocationError),
    #[error("renewal: {0}")]
    Renewal(#[from] RenewError),
}
