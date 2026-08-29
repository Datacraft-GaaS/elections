//! eel-abis-shielding: irreversible shielded template construction and verification.
//!
//! No plaintext template ever leaves the shielded form.
#![warn(missing_docs)]

pub mod params;
pub mod shield;

pub use params::{ShieldingParams, ShieldingParamsError};
pub use shield::{ShieldError, ShieldedTemplate, Shielder};
