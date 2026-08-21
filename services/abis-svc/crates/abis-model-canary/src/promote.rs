//! Canary promotion to 100% traffic.

use chrono::Utc;
use tracing::info;

use crate::domain::{CanaryRecord, CanaryState};
use crate::error::CanaryError;

pub fn promote(record: &mut CanaryRecord) -> Result<(), CanaryError> {
    if record.state != CanaryState::InProgress {
        return Err(CanaryError::NotInProgress);
    }
    record.traffic_pct = 100;
    record.state = CanaryState::Promoted;
    record.promoted_at = Some(Utc::now());
    info!(canary_id = %record.id, "canary promoted");
    Ok(())
}
