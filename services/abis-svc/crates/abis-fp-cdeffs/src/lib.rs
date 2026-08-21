//! CDEFFS: Extended Feature Set for fingerprint records.
//!
//! Reference: NIST SP 500-290 e3 Annex A, "CDEFFS -- Candidate Definition of
//! Extended Feature Set for Fingerprints"; ANSI/NIST-ITL 1-2011 Type-9 fields 300-999.

pub mod features;
pub mod matcher;
pub mod parse;
pub mod serialise;

pub use features::{
    ExtendedFeatureSet, PatternClass, RidgeQualityMap, SingularPoint, SingularPointKind,
};
pub use matcher::CdeffsMatcher;
pub use parse::parse_efs;
pub use serialise::serialise_efs;
