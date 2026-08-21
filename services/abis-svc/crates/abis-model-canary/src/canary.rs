//! Canary lifecycle: start, advance, hold-step logic.

use chrono::{DateTime, Utc};
use tracing::info;
use uuid::Uuid;

use crate::domain::{CanaryRecord, CanaryState, ROLLOUT_STEPS, STEP_HOLD_MINUTES};
use crate::error::CanaryError;

pub fn start_canary(
    from_model_id: Uuid,
    to_model_id: Uuid,
    initial_pct: u8,
    sla_fmr_limit: f64,
    sla_fnmr_limit: f64,
    sla_latency_p99_ms: f64,
    sla_error_rate_limit: f64,
) -> Result<CanaryRecord, CanaryError> {
    if initial_pct > 100 {
        return Err(CanaryError::InvalidTrafficPct(initial_pct));
    }
    let id = Uuid::now_v7();
    let record = CanaryRecord {
        id,
        from_model_id,
        to_model_id,
        traffic_pct: initial_pct,
        state: CanaryState::InProgress,
        sla_fmr_limit,
        sla_fnmr_limit,
        sla_latency_p99_ms,
        sla_error_rate_limit,
        started_at: Utc::now(),
        promoted_at: None,
        rolled_back_at: None,
        rollback_reason: None,
    };
    info!(canary_id = %id, initial_pct, "canary started");
    Ok(record)
}

pub fn next_step(current_pct: u8) -> Option<u8> {
    ROLLOUT_STEPS.iter().copied().find(|&s| s > current_pct)
}

/// Returns the next traffic_pct if the hold period has elapsed, else None.
pub fn try_advance(record: &CanaryRecord, last_step_at: DateTime<Utc>) -> Option<u8> {
    if record.state != CanaryState::InProgress {
        return None;
    }
    let elapsed = Utc::now().signed_duration_since(last_step_at).num_minutes();
    if elapsed < STEP_HOLD_MINUTES {
        return None;
    }
    next_step(record.traffic_pct)
}
