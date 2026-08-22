//! Domain types for model canary rollout.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ROLLOUT_STEPS: &[u8] = &[0, 10, 25, 50, 100];
pub const STEP_HOLD_MINUTES: i64 = 15;

/// Identifies an ABIS model version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCard {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub artifact_uri: String,
}

/// Lifecycle state of a canary deployment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum CanaryState {
    Pending,
    InProgress,
    Promoted,
    RolledBack,
    Cancelled,
}

/// Reason for a rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackReason {
    FmrBreach {
        observed: f64,
        limit: f64,
    },
    FnmrBreach {
        observed: f64,
        limit: f64,
    },
    LatencyBreach {
        p99_ms: f64,
        limit_ms: f64,
    },
    ErrorRateBreach {
        rate: f64,
        limit: f64,
    },
    Manual {
        operator: String,
        reason: String,
    },
    /// HARD-23: canary deviated beyond fairness tolerance on a demographic slice.
    FairnessSliceBreach {
        slice: String,
        metric: String,
        deviation: f64,
        tolerance: f64,
    },
}

/// A canary deployment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryRecord {
    pub id: Uuid,
    pub from_model_id: Uuid,
    pub to_model_id: Uuid,
    pub traffic_pct: u8,
    pub state: CanaryState,
    pub sla_fmr_limit: f64,
    pub sla_fnmr_limit: f64,
    pub sla_latency_p99_ms: f64,
    pub sla_error_rate_limit: f64,
    pub started_at: DateTime<Utc>,
    pub promoted_at: Option<DateTime<Utc>>,
    pub rolled_back_at: Option<DateTime<Utc>>,
    pub rollback_reason: Option<String>,
}

/// Live metrics snapshot for one canary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CanaryMetrics {
    pub fmr: f64,
    pub fnmr: f64,
    pub latency_p99_ms: f64,
    pub error_rate: f64,
    pub adjudication_rate: f64,
    pub sample_count: u64,
}
