//! Bulk deduplication worker.

use crate::domain::FailureRecord;

/// A candidate duplicate pair.
#[derive(Debug, Clone)]
pub struct DuplicatePair {
    pub subject_a: String,
    pub subject_b: String,
    pub score: f64,
}

/// Identify duplicate pairs from a scored pair list above threshold.
pub fn find_duplicates(pairs: &[(String, String, f64)], threshold: f64) -> Vec<DuplicatePair> {
    pairs
        .iter()
        .filter(|(_, _, s)| *s >= threshold)
        .map(|(a, b, s)| DuplicatePair {
            subject_a: a.clone(),
            subject_b: b.clone(),
            score: *s,
        })
        .collect()
}

/// Process a batch: returns failures for pairs that could not be adjudicated.
pub fn process_dedup_batch(
    pairs: &[DuplicatePair],
    fail_ids: &std::collections::HashSet<String>,
) -> Vec<FailureRecord> {
    pairs
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            if fail_ids.contains(&p.subject_a) || fail_ids.contains(&p.subject_b) {
                Some(FailureRecord {
                    item_id: format!("{}:{}", p.subject_a, p.subject_b),
                    error: "dedup adjudication failed".to_string(),
                    offset: i as u64,
                })
            } else {
                None
            }
        })
        .collect()
}
