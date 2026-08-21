//! Progress reporting with failure manifest accumulation.

use crate::domain::{BulkOpState, BulkOperation, FailureRecord, ProgressSnapshot};
use crate::error::BulkOpsError;
use chrono::Utc;
use tracing::{info, warn};

/// Update the progress counters on an operation in place.
pub fn record_progress(op: &mut BulkOperation, snap: ProgressSnapshot) {
    op.checkpoint_offset = snap.offset;
    op.succeeded = snap.succeeded;
    op.failed = snap.failed;
    op.skipped = snap.skipped;
    info!(
        op_id = %op.id,
        offset = snap.offset,
        succeeded = snap.succeeded,
        failed = snap.failed,
        "bulk op progress"
    );
}

/// Mark an operation as completed.
pub fn mark_completed(op: &mut BulkOperation) {
    op.state = BulkOpState::Completed;
    op.completed_at = Some(Utc::now());
    info!(op_id = %op.id, "bulk op completed");
}

/// Mark an operation as failed.
pub fn mark_failed(op: &mut BulkOperation, reason: &str) {
    op.state = BulkOpState::Failed;
    op.completed_at = Some(Utc::now());
    warn!(op_id = %op.id, reason, "bulk op failed");
}

/// Serialise failure records to CSV bytes.
pub fn write_manifest(records: &[FailureRecord]) -> Result<Vec<u8>, BulkOpsError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["item_id", "error", "offset"])?;
    for r in records {
        wtr.write_record([&r.item_id, &r.error, &r.offset.to_string()])?;
    }
    let data = wtr.into_inner().map_err(|e| {
        BulkOpsError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    Ok(data)
}
