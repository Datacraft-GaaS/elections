//! Bulk delete worker (honours data retention policy).

use crate::domain::FailureRecord;

/// Delete a batch of subjects, enforcing retention.
/// Returns (succeeded, failures) for subjects still within retention window.
pub fn process_delete_batch(
    subject_ids: &[String],
    retention_days: u32,
    enrolment_ages_days: &std::collections::HashMap<String, u32>,
) -> (u64, Vec<FailureRecord>) {
    let mut succeeded = 0u64;
    let mut failures = Vec::new();
    for (i, id) in subject_ids.iter().enumerate() {
        let age = enrolment_ages_days.get(id).copied().unwrap_or(0);
        if age < retention_days {
            failures.push(FailureRecord {
                item_id: id.clone(),
                error: format!(
                    "within retention window ({} < {} days)",
                    age, retention_days
                ),
                offset: i as u64,
            });
        } else {
            succeeded += 1;
        }
    }
    (succeeded, failures)
}
