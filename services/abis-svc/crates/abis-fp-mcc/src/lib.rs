//! Minutia Cylinder-Code (MCC) fingerprint matching.
//!
//! Reference: R. Cappelli, M. Ferrara, D. Maltoni, "Minutia cylinder-code: a new
//! representation and matching technique for fingerprint recognition," IEEE Trans.
//! Pattern Anal. Mach. Intell., vol. 32, no. 12, pp. 2128-2141, 2010.

pub mod build;
pub mod consolidate;
pub mod cylinder;
pub mod matcher;
pub mod params;
pub mod similarity;

pub use build::build_mcc;
pub use consolidate::global_similarity;
pub use cylinder::MinutiaCylinder;
pub use matcher::MccMatcher;
pub use params::MccParams;
pub use similarity::local_similarity;
