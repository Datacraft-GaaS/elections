//! Palm biometric modality for the EEL ABIS system.
//!
//! Provides ISO/IEC 19794-2 parsing, Bozorth3-style matching, and zone-weighted
//! quality scoring for full and partial palm prints.

pub mod domain;
pub mod matcher;
pub mod parser;
pub mod quality;

pub use matcher::Score;
