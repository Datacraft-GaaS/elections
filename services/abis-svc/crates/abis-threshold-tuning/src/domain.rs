//! Domain types for threshold tuning.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Biometric modality for which threshold applies.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum BiometricModality {
    Face,
    Fingerprint,
    Iris,
    Voice,
    Multimodal,
}

/// Age band cohort.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgeBand {
    Child,
    YoungAdult,
    Adult,
    Senior,
}

impl AgeBand {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgeBand::Child => "child",
            AgeBand::YoungAdult => "young_adult",
            AgeBand::Adult => "adult",
            AgeBand::Senior => "senior",
        }
    }
}

/// Cohort key fully identifying a threshold profile slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CohortKey {
    pub modality: BiometricModality,
    pub age_band: AgeBand,
    pub sex: String,
    pub county_code: u8,
}

/// A labelled comparison pair (genuine=true, impostor=false).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelledPair {
    pub score: f64,
    pub genuine: bool,
}

/// Bootstrap 95% confidence interval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapCi {
    pub lower: f64,
    pub upper: f64,
    pub n_bootstrap: u32,
}

/// A persisted threshold profile row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdProfile {
    pub id: Uuid,
    pub modality: String,
    pub age_band: String,
    pub sex: String,
    pub county_code: i16,
    pub target_fmr: f64,
    pub threshold: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
    pub n_pairs: i64,
    pub achieved_fmr: f64,
    pub achieved_fnmr: f64,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub note: Option<String>,
}

/// Result of a simulate (dry-run) call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateResult {
    pub threshold: f64,
    pub achieved_fmr: f64,
    pub achieved_fnmr: f64,
    pub ci: BootstrapCi,
    pub n_pairs: usize,
}
