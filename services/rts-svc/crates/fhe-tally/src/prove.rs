//! Public transcript proving correct ciphertext aggregation.
//!
//! Produces a verifiable aggregation transcript: an ordered list of
//! (station_id, slot_hash) pairs plus a running cumulative hash. A third
//! party can re-derive the transcript hash from the ciphertexts without
//! holding any decryption key, verifying that no leaf was dropped or
//! duplicated.
//!
//! This is a lightweight public transcript, not a ZK-SNARK. A full
//! Groth16/Bulletproofs proof is planned in HARD-A item 2.
//!
//! Reference: Ben-Sasson et al. "SNARKs for C: Verifying Program
//! Executions Succinctly and in Zero Knowledge." CRYPTO 2013. (For
//! context on what a stronger proof would entail.)

use serde::{Deserialize, Serialize};

use crate::aggregate::EncryptedTally;
use crate::encrypt_leaf::EncryptedLeaf;
use crate::FheTallyError;

/// A single entry in the aggregation transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub station_id: String,
    /// Blake3 hash of the encrypted slot values for this leaf.
    pub slot_hash: [u8; 32],
}

/// Public aggregation proof: verifiable without decryption keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationProof {
    pub entries: Vec<TranscriptEntry>,
    /// Cumulative Blake3 hash of all entries (chained).
    pub root_hash: [u8; 32],
    pub leaf_count: usize,
}

impl AggregationProof {
    /// Verify that this proof matches the given encrypted tally.
    ///
    /// Checks that: leaf_count matches, and root_hash can be reproduced
    /// from the entries.
    pub fn verify(&self, tally: &EncryptedTally) -> Result<(), FheTallyError> {
        if self.leaf_count != tally.leaf_count {
            return Err(FheTallyError::ProofVerification(format!(
                "leaf count mismatch: proof has {}, tally has {}",
                self.leaf_count, tally.leaf_count
            )));
        }
        let recomputed = compute_root_hash(&self.entries);
        if recomputed != self.root_hash {
            return Err(FheTallyError::ProofVerification(
                "root hash mismatch: entries do not match claimed root".into(),
            ));
        }
        Ok(())
    }
}

/// Build a proof from the ordered set of encrypted leaves.
pub fn build_aggregation_proof(
    leaves: &[EncryptedLeaf],
    tally: &EncryptedTally,
) -> Result<AggregationProof, FheTallyError> {
    if leaves.is_empty() {
        return Err(FheTallyError::Aggregation("no leaves for proof".into()));
    }
    if leaves.len() != tally.leaf_count {
        return Err(FheTallyError::ProofVerification(format!(
            "leaf count mismatch: {} leaves but tally has {}",
            leaves.len(),
            tally.leaf_count
        )));
    }

    let entries: Vec<TranscriptEntry> = leaves
        .iter()
        .map(|leaf| {
            let mut h = blake3::Hasher::new();
            for &slot in &leaf.slots {
                h.update(&slot.to_le_bytes());
            }
            TranscriptEntry {
                station_id: leaf.station_id.clone(),
                slot_hash: *h.finalize().as_bytes(),
            }
        })
        .collect();

    let root_hash = compute_root_hash(&entries);

    Ok(AggregationProof {
        entries,
        root_hash,
        leaf_count: tally.leaf_count,
    })
}

fn compute_root_hash(entries: &[TranscriptEntry]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for entry in entries {
        hasher.update(entry.station_id.as_bytes());
        hasher.update(&entry.slot_hash);
    }
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::aggregate_encrypted_leaves;
    use crate::encrypt_leaf::{encrypt_34a_leaf, Form34ALeaf};

    fn make_leaves(n: usize) -> Vec<EncryptedLeaf> {
        (0..n)
            .map(|i| {
                let l = Form34ALeaf {
                    station_id: format!("s{i}"),
                    candidate_counts: vec![
                        ("c0".into(), (i * 10) as u64),
                        ("c1".into(), (i * 5) as u64),
                    ],
                };
                encrypt_34a_leaf(&l, 1.0).unwrap()
            })
            .collect()
    }

    #[test]
    fn proof_verifies_correctly() {
        let leaves = make_leaves(3);
        let tally = aggregate_encrypted_leaves(&leaves).unwrap();
        let proof = build_aggregation_proof(&leaves, &tally).unwrap();
        proof.verify(&tally).unwrap();
    }

    #[test]
    fn proof_detects_tampered_root_hash() {
        let leaves = make_leaves(3);
        let tally = aggregate_encrypted_leaves(&leaves).unwrap();
        let mut proof = build_aggregation_proof(&leaves, &tally).unwrap();
        proof.root_hash[0] ^= 0xFF;
        let err = proof.verify(&tally).unwrap_err();
        assert!(matches!(err, FheTallyError::ProofVerification(_)));
    }

    #[test]
    fn proof_detects_leaf_count_mismatch() {
        let leaves = make_leaves(3);
        let tally = aggregate_encrypted_leaves(&leaves).unwrap();
        let mut proof = build_aggregation_proof(&leaves, &tally).unwrap();
        proof.leaf_count = 99;
        let err = proof.verify(&tally).unwrap_err();
        assert!(matches!(err, FheTallyError::ProofVerification(_)));
    }

    #[test]
    fn root_hash_deterministic() {
        let leaves = make_leaves(2);
        let tally = aggregate_encrypted_leaves(&leaves).unwrap();
        let p1 = build_aggregation_proof(&leaves, &tally).unwrap();
        let p2 = build_aggregation_proof(&leaves, &tally).unwrap();
        assert_eq!(p1.root_hash, p2.root_hash);
    }
}
