//! `eel-abis-perf-monitor`: continuous FMR/FNMR + rank-N accuracy monitoring.
//!
//! Recomputes metrics on a labelled holdout set every 6 hours.
//! Produces daily, weekly, and monthly rollup reports.

pub mod domain;
pub mod error;
pub mod pmr;
pub mod report;
pub mod target;

pub use domain::{HoldoutPair, PerformanceReport, PerformanceTarget, RollupPeriod};
pub use error::PerfMonitorError;
pub use pmr::{compute_fmr, compute_fnmr, compute_rank_n_accuracy, PmrResult};
