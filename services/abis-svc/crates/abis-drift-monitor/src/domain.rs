//! Domain types for drift monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Rolling window sizes for drift computation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftWindow {
    H24,
    D7,
    D30,
}

impl DriftWindow {
    pub fn hours(&self) -> i64 {
        match self {
            DriftWindow::H24 => 24,
            DriftWindow::D7 => 7 * 24,
            DriftWindow::D30 => 30 * 24,
        }
    }
}

/// Severity levels for drift alerts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DriftSeverity {
    Info,
    Warning,
    Critical,
}

/// A snapshot of a feature's empirical distribution as histogram bins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDistribution {
    pub feature_name: String,
    pub bin_counts: Vec<u64>,
    pub n_total: u64,
    pub captured_at: DateTime<Utc>,
}

impl FeatureDistribution {
    /// Normalise bin_counts to probabilities with Laplace smoothing to avoid zeros.
    pub fn to_probs(&self) -> Vec<f64> {
        let n = self.n_total as f64 + self.bin_counts.len() as f64;
        self.bin_counts
            .iter()
            .map(|&c| (c as f64 + 1.0) / n)
            .collect()
    }
}

/// A drift measurement report for one feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub feature_name: String,
    pub window: DriftWindow,
    pub psi: f64,
    pub ks_stat: f64,
    pub js_div: f64,
    pub wasserstein: f64,
    pub severity: DriftSeverity,
    pub computed_at: DateTime<Utc>,
}

/// A persisted drift alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAlert {
    pub id: Uuid,
    pub model_id: Uuid,
    pub feature_name: String,
    pub severity: DriftSeverity,
    pub psi: f64,
    pub ks_stat: f64,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}
