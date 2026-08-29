//! NIST FRVT evaluation harness.
//!
//! Reference: NIST FRVT Ongoing (frvt.nist.gov). Phillips, P. J. et al.
//! (2018). "An Introduction to the Good, the Bad, & the Ugly Face
//! Recognition Challenge Problem." FG 2011. (FRVT metric definitions)
//!
//! Metrics implemented:
//! - 1:1 verification: FMR (false match rate), FNMR (false non-match rate)
//!   at a decision threshold.
//! - 1:N identification: Rank-1 through Rank-N.
//! - Quality track: correlation between quality score and FNMR.
//! - Per-demographic fairness slices: FMR/FNMR by age band, sex, skin tone.
#![warn(missing_docs)]

pub mod corpus;
pub mod metrics;
pub mod report;

pub use corpus::{FrvtCorpus, FrvtImage, FrvtImageSet, SkinTone, Sex};
pub use metrics::{FmrFnmr, RankN, QualityCorrelation, FairnessSlice};
pub use report::FrvtReport;
