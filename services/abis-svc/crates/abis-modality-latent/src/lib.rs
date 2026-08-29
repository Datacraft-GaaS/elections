//! Latent biometric modality for the EEL ABIS system.
//!
//! Provides ANSI/NIST-ITL Type-13 parsing, quality-weighted partial print
//! matching, chain-of-custody management, and quality scoring for latent prints.
#![warn(missing_docs)]

pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use matcher::Score;
