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

//! Bloom-filter-based shielded template.
//!
//! A shielded template is a Bloom filter populated from template features.
//! Verification uses intersection-based similarity without ever exposing the
//! plaintext template.

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShieldError {
    #[error("template must not be empty")]
    EmptyTemplate,
    #[error("filter_size_bits must be >= 64")]
    FilterTooSmall,
    #[error("hash_count must be in [1, 32]")]
    InvalidHashCount,
}

/// A Bloom-filter shielded biometric template.
#[derive(Debug, Clone)]
pub struct ShieldedTemplate {
    /// Bit array stored as bytes.
    pub filter: Vec<u8>,
    pub filter_size_bits: usize,
    pub hash_count: u8,
}

impl ShieldedTemplate {
    /// Computes Jaccard-like intersection similarity with another shielded template.
    ///
    /// Returns a similarity score in [0, 1].
    pub fn similarity(&self, other: &ShieldedTemplate) -> f64 {
        if self.filter.len() != other.filter.len() {
            return 0.0;
        }
        let intersection: usize = self
            .filter
            .iter()
            .zip(other.filter.iter())
            .map(|(a, b)| (a & b).count_ones() as usize)
            .sum();
        let union: usize = self
            .filter
            .iter()
            .zip(other.filter.iter())
            .map(|(a, b)| (a | b).count_ones() as usize)
            .sum();
        if union == 0 {
            return 1.0;
        }
        intersection as f64 / union as f64
    }

    /// Returns the popcount (number of set bits).
    pub fn popcount(&self) -> usize {
        self.filter.iter().map(|b| b.count_ones() as usize).sum()
    }
}

/// Constructs shielded templates from plaintext feature byte slices.
pub struct Shielder {
    filter_size_bits: usize,
    hash_count: u8,
}

impl Shielder {
    pub fn new(filter_size_bits: usize, hash_count: u8) -> Result<Self, ShieldError> {
        if filter_size_bits < 64 {
            return Err(ShieldError::FilterTooSmall);
        }
        if hash_count < 1 || hash_count > 32 {
            return Err(ShieldError::InvalidHashCount);
        }
        Ok(Self {
            filter_size_bits,
            hash_count,
        })
    }

    /// Shields a plaintext template into a Bloom filter.
    ///
    /// The plaintext template is processed locally and never stored.
    ///
    /// # Errors
    /// Returns `ShieldError::EmptyTemplate` if the feature slice is empty.
    pub fn shield(&self, template: &[u8]) -> Result<ShieldedTemplate, ShieldError> {
        if template.is_empty() {
            return Err(ShieldError::EmptyTemplate);
        }

        let byte_count = (self.filter_size_bits + 7) / 8;
        let mut filter = vec![0u8; byte_count];

        for h_idx in 0..self.hash_count {
            let mut hasher = Sha256::new();
            hasher.update(&[h_idx]);
            hasher.update(template);
            let digest = hasher.finalize();

            // Extract a bit position from the digest.
            let pos = u64::from_le_bytes(digest[..8].try_into().unwrap_or([0u8; 8])) as usize
                % self.filter_size_bits;
            filter[pos / 8] |= 1 << (pos % 8);
        }

        Ok(ShieldedTemplate {
            filter,
            filter_size_bits: self.filter_size_bits,
            hash_count: self.hash_count,
        })
    }
}
