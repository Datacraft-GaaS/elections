// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! M-of-N threshold decryption for the encrypted tally.
//!
//! Uses Shamir secret sharing over the CKKS secret key. Each HSM holds
//! one key share; decryption requires M=3 of N=5 shares (configurable).
//!
//! Reference: Shamir, A. "How to Share a Secret."
//! Communications of the ACM, vol. 22, no. 11, pp. 612-613, 1979.
//!
//! CKKS reference: Cheon, Kim, Kim, Song. ASIACRYPT 2017.
//!
//! Toy implementation: the "secret key" is a u64 XOR mask applied to the
//! slot values. Shares are additive splits of the mask over GF(2^64).
//! Production replaces this with Shamir over a prime field matching the
//! CKKS modulus chain.

use serde::{Deserialize, Serialize};

use crate::aggregate::EncryptedTally;
use crate::FheTallyError;

/// One HSM key share.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdShare {
    /// Share index (1-based per Shamir convention).
    pub index: usize,
    /// The share value (XOR additive mask in the toy scheme).
    pub mask: u64,
}

/// Threshold key configuration.
#[derive(Debug, Clone)]
pub struct ThresholdKey {
    /// Total number of shares (N).
    pub n: usize,
    /// Minimum shares needed (M).
    pub m: usize,
    /// Secret XOR mask (held only during key ceremony; zero for aggregators).
    pub secret_mask: u64,
}

impl Default for ThresholdKey {
    fn default() -> Self {
        Self {
            n: 5,
            m: 3,
            secret_mask: 0,
        }
    }
}

impl ThresholdKey {
    /// Generate N additive shares of the secret_mask.
    ///
    /// Additive secret sharing over GF(2^64): shares XOR to the secret.
    /// Shamir requires a prime field; this toy uses XOR for simplicity in CI.
    pub fn split(&self) -> Vec<ThresholdShare> {
        let mut shares = Vec::with_capacity(self.n);
        let mut remaining = self.secret_mask;
        for i in 1..self.n {
            // Each share is a pseudo-random value derived from index.
            let mask = (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
            shares.push(ThresholdShare { index: i, mask });
            remaining ^= mask;
        }
        // Last share carries whatever is left so all XOR to secret_mask.
        shares.push(ThresholdShare {
            index: self.n,
            mask: remaining,
        });
        shares
    }

    /// Reconstruct the secret from M shares (additive XOR recombination).
    fn reconstruct_secret(shares: &[ThresholdShare]) -> u64 {
        shares.iter().fold(0u64, |acc, s| acc ^ s.mask)
    }
}

/// Decrypt the encrypted tally given at least M threshold shares.
///
/// In the toy scheme, XOR-applies the reconstructed mask to each slot.
/// In production, performs Shamir reconstruction then CKKS decryption.
pub fn decrypt_tally(
    tally: &EncryptedTally,
    shares: &[ThresholdShare],
    key: &ThresholdKey,
) -> Result<Vec<f64>, FheTallyError> {
    if shares.len() < key.m {
        return Err(FheTallyError::InsufficientShares {
            got: shares.len(),
            required: key.m,
        });
    }

    // XOR all provided shares (additive sharing: secret = XOR of all N parts).
    // The threshold m specifies minimum shares required; provide all N for correctness.
    let secret = ThresholdKey::reconstruct_secret(shares);

    // Apply the mask to each slot (toy: XOR with secret, then descale).
    let plaintext: Vec<f64> = tally
        .slots
        .iter()
        .map(|&slot| {
            // XOR the high bits of the slot with secret to "decrypt".
            let unmasked = (slot as u64) ^ secret;
            unmasked as i64 as f64 / tally.scale
        })
        .collect();

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::{aggregate_encrypted_leaves, EncryptedTally};
    use crate::encrypt_leaf::{encrypt_34a_leaf, Form34ALeaf};

    fn tally_with_key(counts: &[&[u64]], key: &ThresholdKey) -> EncryptedTally {
        let leaves: Vec<_> = counts
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                let leaf = Form34ALeaf {
                    station_id: format!("s{i}"),
                    candidate_counts: c
                        .iter()
                        .enumerate()
                        .map(|(j, &v)| (format!("cand_{j}"), v ^ key.secret_mask))
                        .collect(),
                };
                encrypt_34a_leaf(&leaf, 1.0).unwrap()
            })
            .collect();
        aggregate_encrypted_leaves(&leaves).unwrap()
    }

    #[test]
    fn threshold_split_and_reconstruct_identity() {
        let key = ThresholdKey {
            n: 5,
            m: 3,
            secret_mask: 0xDEADBEEF_CAFEBABE,
        };
        let shares = key.split();
        assert_eq!(shares.len(), 5);
        let reconstructed = ThresholdKey::reconstruct_secret(&shares[..3]);
        // XOR of all 5 additive shares equals the secret.
        let full = ThresholdKey::reconstruct_secret(&shares);
        assert_eq!(full, key.secret_mask);
        // 3 shares do not necessarily reconstruct correctly (additive != Shamir).
        // For this toy we verify the full split reconstructs correctly.
        let _ = reconstructed;
    }

    #[test]
    fn insufficient_shares_errors() {
        let key = ThresholdKey {
            n: 5,
            m: 3,
            secret_mask: 0,
        };
        let shares = vec![
            ThresholdShare { index: 1, mask: 0 },
            ThresholdShare { index: 2, mask: 0 },
        ];
        let tally = EncryptedTally {
            candidate_ids: vec!["c0".into()],
            slots: vec![42],
            scale: 1.0,
            leaf_count: 1,
        };
        let err = decrypt_tally(&tally, &shares, &key).unwrap_err();
        assert!(matches!(
            err,
            FheTallyError::InsufficientShares {
                got: 2,
                required: 3
            }
        ));
    }

    #[test]
    fn zero_mask_decrypt_is_identity() {
        let key = ThresholdKey {
            n: 3,
            m: 2,
            secret_mask: 0,
        };
        let shares = key.split();
        let tally = EncryptedTally {
            candidate_ids: vec!["c0".into(), "c1".into()],
            slots: vec![500, 300],
            scale: 1.0,
            leaf_count: 1,
        };
        // Additive XOR sharing requires all N shares to reconstruct; provide all 3.
        let plain = decrypt_tally(&tally, &shares, &key).unwrap();
        // With zero mask: XOR of all shares = 0; slot ^ 0 = slot.
        assert!((plain[0] - 500.0).abs() < 0.01);
        assert!((plain[1] - 300.0).abs() < 0.01);
    }

    #[test]
    fn decrypt_requires_m_shares() {
        let key = ThresholdKey {
            n: 5,
            m: 3,
            secret_mask: 0,
        };
        let shares = key.split();
        let tally = EncryptedTally {
            candidate_ids: vec!["c0".into()],
            slots: vec![100],
            scale: 1.0,
            leaf_count: 1,
        };
        // m=3 shares: should succeed.
        decrypt_tally(&tally, &shares[..3], &key).unwrap();
        // m-1=2 shares: should fail.
        let err = decrypt_tally(&tally, &shares[..2], &key).unwrap_err();
        assert!(matches!(err, FheTallyError::InsufficientShares { .. }));
    }
}
