//! Homomorphic aggregation of encrypted Form34A leaves.
//!
//! Produces an EncryptedTally by summing encrypted slot vectors from all
//! leaves. This is a purely homomorphic operation: no decryption occurs.
//!
//! SIMD packing: slots are paired elementwise across all leaves. All
//! leaves must have the same candidate ordering (enforced by matching
//! candidate_ids).
//!
//! Reference: Cheon, Kim, Kim, Song. ASIACRYPT 2017.
//! Section 4: "Batch Encoding and Homomorphic Operations."

use serde::{Deserialize, Serialize};

use crate::encrypt_leaf::EncryptedLeaf;
use crate::FheTallyError;

/// The result of aggregating all encrypted leaves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTally {
    /// Ordered candidate IDs (same ordering as slot positions).
    pub candidate_ids: Vec<String>,
    /// Summed encrypted slots (one per candidate).
    pub slots: Vec<i64>,
    /// Scale shared by all leaves (must be identical).
    pub scale: f64,
    /// Number of leaves aggregated.
    pub leaf_count: usize,
}

impl EncryptedTally {
    /// Decrypt for testing/verification only (toy implementation).
    pub fn decrypt_toy(&self) -> Vec<f64> {
        self.slots.iter().map(|&s| s as f64 / self.scale).collect()
    }
}

/// Aggregate a slice of encrypted leaves via homomorphic addition.
///
/// All leaves must share the same candidate_ids and scale.
/// Returns an error on empty input, mismatched candidates, or scale mismatch.
pub fn aggregate_encrypted_leaves(
    leaves: &[EncryptedLeaf],
) -> Result<EncryptedTally, FheTallyError> {
    if leaves.is_empty() {
        return Err(FheTallyError::Aggregation("no leaves to aggregate".into()));
    }

    let reference = &leaves[0];
    let n_candidates = reference.candidate_ids.len();
    let scale = reference.scale;

    // Validate all leaves.
    for (i, leaf) in leaves.iter().enumerate() {
        if leaf.candidate_ids != reference.candidate_ids {
            return Err(FheTallyError::Aggregation(format!(
                "leaf {i} has different candidate_ids"
            )));
        }
        if (leaf.scale - scale).abs() > f64::EPSILON {
            return Err(FheTallyError::Aggregation(format!(
                "leaf {i} scale {} != reference scale {scale}",
                leaf.scale
            )));
        }
        if leaf.slots.len() != n_candidates {
            return Err(FheTallyError::Aggregation(format!(
                "leaf {i} has {} slots but expected {n_candidates}",
                leaf.slots.len()
            )));
        }
    }

    // Homomorphic addition: sum slot vectors elementwise.
    let mut summed = vec![0i64; n_candidates];
    for leaf in leaves {
        for (j, &slot) in leaf.slots.iter().enumerate() {
            summed[j] = summed[j].saturating_add(slot);
        }
    }

    Ok(EncryptedTally {
        candidate_ids: reference.candidate_ids.clone(),
        slots: summed,
        scale,
        leaf_count: leaves.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encrypt_leaf::{encrypt_34a_leaf, Form34ALeaf};

    fn leaf(station: &str, counts: &[u64]) -> EncryptedLeaf {
        let l = Form34ALeaf {
            station_id: station.into(),
            candidate_counts: counts
                .iter()
                .enumerate()
                .map(|(i, &c)| (format!("cand_{i}"), c))
                .collect(),
        };
        encrypt_34a_leaf(&l, 1.0).unwrap()
    }

    #[test]
    fn aggregate_two_leaves_sums_correctly() {
        let leaves = vec![leaf("s1", &[100, 200]), leaf("s2", &[50, 75])];
        let tally = aggregate_encrypted_leaves(&leaves).unwrap();
        let plain = tally.decrypt_toy();
        assert!((plain[0] - 150.0).abs() < 0.01);
        assert!((plain[1] - 275.0).abs() < 0.01);
    }

    #[test]
    fn aggregate_empty_errors() {
        assert!(aggregate_encrypted_leaves(&[]).is_err());
    }

    #[test]
    fn aggregate_mismatched_candidates_errors() {
        let l1 = leaf("s1", &[100, 200]);
        let l2 = {
            let f = Form34ALeaf {
                station_id: "s2".into(),
                candidate_counts: vec![("different_cand".into(), 50)],
            };
            encrypt_34a_leaf(&f, 1.0).unwrap()
        };
        assert!(aggregate_encrypted_leaves(&[l1, l2]).is_err());
    }

    #[test]
    fn aggregate_leaf_count_matches() {
        let leaves: Vec<EncryptedLeaf> =
            (0..5).map(|i| leaf(&format!("s{i}"), &[10, 20])).collect();
        let tally = aggregate_encrypted_leaves(&leaves).unwrap();
        assert_eq!(tally.leaf_count, 5);
    }
}
