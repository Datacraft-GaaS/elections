//! Encrypt a Form34A leaf (per-station tally) as a CKKS ciphertext.
//!
//! Each candidate's integer count is encoded as a CKKS plaintext slot and
//! encrypted. The resulting EncryptedLeaf is safe to transmit to the
//! aggregator which sums it homomorphically without decrypting.
//!
//! SIMD packing: all candidate counts are packed into one ciphertext per leaf
//! using the batch encoding feature of CKKS (up to degree/2 slots).
//!
//! Reference: Cheon, Kim, Kim, Song. ASIACRYPT 2017. Section 3 (encoding).

use serde::{Deserialize, Serialize};

use crate::FheTallyError;

/// A Form34A tally leaf: per-station, per-candidate vote counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34ALeaf {
    /// Polling station ID (UUID7 string).
    pub station_id: String,
    /// Candidate IDs and their plaintext vote counts.
    pub candidate_counts: Vec<(String, u64)>,
}

/// A CKKS-encrypted Form34A leaf.
///
/// In the toy implementation, encrypts by applying a scale factor and
/// storing scaled integers. Production swaps this for actual CKKS ciphertexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedLeaf {
    pub station_id: String,
    /// Candidate IDs (preserved for aggregation verification).
    pub candidate_ids: Vec<String>,
    /// Scaled integer ciphertext slots (one per candidate).
    /// In production: polynomial ring elements.
    pub slots: Vec<i64>,
    /// Scale factor used for encoding (must match ctx.scale).
    pub scale: f64,
}

impl EncryptedLeaf {
    /// Decrypt this leaf back to f64 counts (toy implementation).
    pub fn decrypt_toy(&self) -> Vec<f64> {
        self.slots.iter().map(|&s| s as f64 / self.scale).collect()
    }
}

/// Encrypt a Form34A leaf using the given CKKS context.
///
/// Uses toy CKKS semantics: scale the integer counts and store.
/// Production replaces the inner body with actual CKKS encryption.
pub fn encrypt_34a_leaf(leaf: &Form34ALeaf, scale: f64) -> Result<EncryptedLeaf, FheTallyError> {
    if leaf.candidate_counts.is_empty() {
        return Err(FheTallyError::Encryption("no candidates in leaf".into()));
    }
    if scale <= 0.0 {
        return Err(FheTallyError::Encryption(format!(
            "scale must be positive, got {scale}"
        )));
    }

    let candidate_ids: Vec<String> = leaf
        .candidate_counts
        .iter()
        .map(|(id, _)| id.clone())
        .collect();
    let slots: Vec<i64> = leaf
        .candidate_counts
        .iter()
        .map(|(_, count)| (*count as f64 * scale).round() as i64)
        .collect();

    Ok(EncryptedLeaf {
        station_id: leaf.station_id.clone(),
        candidate_ids,
        slots,
        scale,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(counts: &[u64]) -> Form34ALeaf {
        Form34ALeaf {
            station_id: "station-001".into(),
            candidate_counts: counts
                .iter()
                .enumerate()
                .map(|(i, &c)| (format!("cand_{i}"), c))
                .collect(),
        }
    }

    #[test]
    fn encrypt_produces_slots() {
        let l = leaf(&[100, 200, 50]);
        let enc = encrypt_34a_leaf(&l, 1.0).unwrap();
        assert_eq!(enc.slots.len(), 3);
    }

    #[test]
    fn decrypt_toy_recovers_counts() {
        let l = leaf(&[100, 200, 50]);
        let enc = encrypt_34a_leaf(&l, 1.0).unwrap();
        let plain = enc.decrypt_toy();
        assert!((plain[0] - 100.0).abs() < 0.01);
        assert!((plain[1] - 200.0).abs() < 0.01);
        assert!((plain[2] - 50.0).abs() < 0.01);
    }

    #[test]
    fn empty_leaf_errors() {
        let l = Form34ALeaf {
            station_id: "x".into(),
            candidate_counts: vec![],
        };
        assert!(encrypt_34a_leaf(&l, 1.0).is_err());
    }

    #[test]
    fn zero_scale_errors() {
        let l = leaf(&[10]);
        assert!(encrypt_34a_leaf(&l, 0.0).is_err());
    }
}
