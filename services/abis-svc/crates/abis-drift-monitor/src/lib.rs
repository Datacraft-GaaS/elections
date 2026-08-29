//! `eel-abis-drift-monitor`: feature-space drift detection and alerting.
//!
//! Computes PSI, KS statistic, Jensen-Shannon divergence, and Wasserstein distance
//! across configurable rolling windows (24h, 7d, 30d) against a baseline snapshot.
#![warn(missing_docs)]

pub mod adaptive;
pub mod alert;
pub mod detector;
pub mod domain;
pub mod error;
pub mod window;

pub use detector::{js_divergence, ks_statistic, psi, wasserstein_distance, DriftDetector};
pub use domain::{DriftAlert, DriftReport, DriftSeverity, DriftWindow, FeatureDistribution};
pub use error::DriftError;
