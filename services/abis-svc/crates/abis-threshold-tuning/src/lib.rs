//! `eel-abis-threshold-tuning`: per-cohort ROC-based threshold selection with 95% bootstrap CIs.
//!
//! Selects decision thresholds that achieve a target false match rate (FMR),
//! partitioned by modality, age band, sex, and county cohort. Bootstrap resampling
//! (n=1000) produces 95% confidence intervals.

pub mod activate;
pub mod api;
pub mod domain;
pub mod error;
pub mod repo;
pub mod tuner;

pub use domain::{
	AgeBand, BiometricModality, BootstrapCi, CohortKey, LabelledPair, SimulateResult,
	ThresholdProfile,
};
pub use error::ThresholdError;
pub use tuner::{compute_roc, select_threshold, ThresholdTuner};
