//! Bulk re-wrapping of biometric templates under a new KEK.

use crate::domain::FailureRecord;

/// Simulate KEK re-wrapping for a batch of template IDs.
pub fn process_rewrap_batch(
    template_ids: &[String],
    old_kek_id: &str,
    new_kek_id: &str,
    fail_ids: &std::collections::HashSet<String>,
) -> (u64, Vec<FailureRecord>) {
    let _ = (old_kek_id, new_kek_id);
    let mut succeeded = 0u64;
    let mut failures = Vec::new();
    for (i, tid) in template_ids.iter().enumerate() {
        if fail_ids.contains(tid) {
            failures.push(FailureRecord {
                item_id: tid.clone(),
                error: "rewrap failed".to_string(),
                offset: i as u64,
            });
        } else {
            succeeded += 1;
        }
    }
    (succeeded, failures)
}
