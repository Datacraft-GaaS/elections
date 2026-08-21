//! Bulk retirement: soft-delete subjects from active gallery.

use crate::domain::FailureRecord;
use chrono::{DateTime, Utc};

/// A retirement record.
#[derive(Debug, Clone)]
pub struct RetirementRecord {
    pub subject_id: String,
    pub retired_at: DateTime<Utc>,
    pub reason: String,
}

/// Soft-retire a batch of subjects.
pub fn process_retire_batch(
    subject_ids: &[String],
    reason: &str,
    fail_ids: &std::collections::HashSet<String>,
) -> (Vec<RetirementRecord>, Vec<FailureRecord>) {
    let mut retired = Vec::new();
    let mut failures = Vec::new();
    for (i, id) in subject_ids.iter().enumerate() {
        if fail_ids.contains(id) {
            failures.push(FailureRecord {
                item_id: id.clone(),
                error: "retirement rejected".to_string(),
                offset: i as u64,
            });
        } else {
            retired.push(RetirementRecord {
                subject_id: id.clone(),
                retired_at: Utc::now(),
                reason: reason.to_string(),
            });
        }
    }
    (retired, failures)
}
