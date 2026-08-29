//! `eel-abis-twin-registry`: twin group management with civil-registry sync.
//!
//! Identical twins score in an ambiguous zone during 1:N matching. This crate
//! manages twin groups, detects twin candidates, and cross-references civil registry.
#![warn(missing_docs)]

pub mod api;
pub mod detector;
pub mod domain;
pub mod error;
pub mod registry;

pub use domain::{TwinCandidate, TwinGroup, TwinMember, TwinVerificationStatus};
pub use error::TwinRegistryError;
