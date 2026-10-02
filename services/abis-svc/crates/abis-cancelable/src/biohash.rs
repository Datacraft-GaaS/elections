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

//! BioHashing: irreversible transformation of feature vectors using a user key.
//!
//! Feature vector x user-key -> fixed-length bit string.
//! Revocation is achieved by rotating the user key.

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BioHashError {
    #[error("feature vector must not be empty")]
    EmptyFeature,
    #[error("user key must not be empty")]
    EmptyKey,
    #[error("bit_count must be in [8, 4096]")]
    InvalidBitCount,
}

/// A BioHash is a fixed-length bit vector stored as a byte vector (MSB-first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BioHash {
    pub bits: Vec<u8>,
    pub bit_count: usize,
}

impl BioHash {
    /// Hamming distance between two BioHashes of equal length.
    ///
    /// Returns `None` when lengths differ.
    pub fn hamming_distance(&self, other: &BioHash) -> Option<usize> {
        if self.bits.len() != other.bits.len() {
            return None;
        }
        Some(
            self.bits
                .iter()
                .zip(other.bits.iter())
                .map(|(a, b)| (a ^ b).count_ones() as usize)
                .sum(),
        )
    }
}

/// Computes BioHashes from feature vectors and user keys.
pub struct BioHasher {
    bit_count: usize,
}

impl BioHasher {
    pub fn new(bit_count: usize) -> Result<Self, BioHashError> {
        if !(8..=4096).contains(&bit_count) {
            return Err(BioHashError::InvalidBitCount);
        }
        Ok(Self { bit_count })
    }

    /// Transforms a feature vector with a user-specific key into an irreversible hash.
    ///
    /// The transformation is one-way: inverting requires brute-force over the
    /// key space, which for a 256-bit key requires >= 2^256 operations.
    ///
    /// # Errors
    /// Returns an error if the feature vector or key is empty.
    pub fn hash(&self, feature: &[f32], user_key: &[u8]) -> Result<BioHash, BioHashError> {
        if feature.is_empty() {
            return Err(BioHashError::EmptyFeature);
        }
        if user_key.is_empty() {
            return Err(BioHashError::EmptyKey);
        }

        let byte_count = (self.bit_count + 7) / 8;
        let mut bits = Vec::with_capacity(byte_count);

        // Derive bit positions from SHA-256(key || feature_index).
        for chunk_idx in 0..byte_count {
            let mut h = Sha256::new();
            h.update(user_key);
            h.update(&(chunk_idx as u64).to_le_bytes());
            // Mix feature values into the hash.
            for (fi, fv) in feature.iter().enumerate() {
                h.update(&(fi as u32).to_le_bytes());
                h.update(&fv.to_le_bytes());
            }
            let digest = h.finalize();
            bits.push(digest[0]);
        }

        bits.truncate(byte_count);
        Ok(BioHash {
            bits,
            bit_count: self.bit_count,
        })
    }
}
