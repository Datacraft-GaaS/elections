//! Canary rollback: manual and automatic paths.

use chrono::Utc;
use tracing::warn;

use crate::domain::{CanaryRecord, CanaryState, RollbackReason};
use crate::error::CanaryError;

/// Roll back a canary to 0% traffic. Idempotent if already rolled back.
pub fn rollback(record: &mut CanaryRecord, reason: RollbackReason) -> Result<(), CanaryError> {
    if record.state == CanaryState::RolledBack {
        return Ok(());
    }
    if record.state != CanaryState::InProgress {
        return Err(CanaryError::NotInProgress);
    }
    let reason_json = serde_json::to_string(&reason).unwrap_or_else(|_| "unknown".to_string());
    warn!(canary_id = %record.id, reason = %reason_json, "canary rolled back");
    record.traffic_pct = 0;
    record.state = CanaryState::RolledBack;
    record.rolled_back_at = Some(Utc::now());
    record.rollback_reason = Some(reason_json);
    Ok(())
}
