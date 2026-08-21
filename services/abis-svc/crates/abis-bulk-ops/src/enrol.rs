//! Bulk enrolment worker.

use crate::domain::{BulkOperation, FailureRecord};
use tracing::debug;

/// Simulated subject record for enrolment.
#[derive(Debug, Clone)]
pub struct SubjectRecord {
    pub id: String,
    pub payload: Vec<u8>,
}

/// Process a batch of subject enrolment records.
/// Returns (succeeded, failures).
pub fn process_enrol_batch(
    op: &BulkOperation,
    subjects: &[SubjectRecord],
    offset: u64,
    fail_ids: &std::collections::HashSet<String>,
) -> (u64, Vec<FailureRecord>) {
    let mut succeeded = 0u64;
    let mut failures = Vec::new();
    for (i, s) in subjects.iter().enumerate() {
        if fail_ids.contains(&s.id) {
            failures.push(FailureRecord {
                item_id: s.id.clone(),
                error: "enrolment rejected".to_string(),
                offset: offset + i as u64,
            });
        } else {
            succeeded += 1;
            debug!(op_id = %op.id, subject_id = %s.id, "enrolled");
        }
    }
    (succeeded, failures)
}
