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

//! Demographic Bloom-filter pruner for 1:N cascade.
//!
//! Maintains one Bloom filter per (county, sex, age-bucket) demographic slice.
//! Subjects are added to their slice's filter; probe lookup returns only the
//! slices that likely contain the subject.
//!
//! Filter spec: 8 bits-per-key, 0.01 FPR target, double-hashing with FNV1a + FNV1b.

use std::collections::HashMap;

/// Sex discriminant for demographic bucketing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    M,
    F,
}

/// Age bucket per electoral-register ISO convention.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgeBucket {
    Age18_25,
    Age26_40,
    Age41_60,
    Age61Plus,
}

impl AgeBucket {
    pub fn from_age(age: u8) -> Self {
        match age {
            0..=25 => Self::Age18_25,
            26..=40 => Self::Age26_40,
            41..=60 => Self::Age41_60,
            _ => Self::Age61Plus,
        }
    }
}

/// Demographic slice key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DemographicKey {
    pub sex: Sex,
    pub age_bucket: AgeBucket,
    pub county: String,
}

// ---------------------------------------------------------------------------
// Minimal Bloom filter (~150 lines total including this comment block).
// Double-hashing: h(i) = (h1 + i * h2) mod m.
// h1 = FNV1a-64, h2 = FNV1b-64 variant.
// ---------------------------------------------------------------------------

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;
const K_HASHES: usize = 6; // k=6 optimal for ~0.01 FPR at 8 bpk

fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Slightly varied seed for the second hash function.
fn fnv1b_64(data: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS ^ 0xABCD_EF01_2345_6789;
    for &b in data {
        hash = hash.wrapping_mul(FNV_PRIME);
        hash ^= b as u64;
    }
    hash
}

struct BloomFilter {
    bits: Vec<u64>,
    num_bits: usize,
}

impl BloomFilter {
    /// Allocate a Bloom filter for `capacity` items at 8 bits-per-key.
    fn new(capacity: usize) -> Self {
        let num_bits = (capacity * 8).max(64);
        let num_words = num_bits.div_ceil(64);
        Self {
            bits: vec![0u64; num_words],
            num_bits,
        }
    }

    fn set_bit(&mut self, pos: usize) {
        self.bits[pos / 64] |= 1u64 << (pos % 64);
    }

    fn get_bit(&self, pos: usize) -> bool {
        (self.bits[pos / 64] >> (pos % 64)) & 1 == 1
    }

    fn insert(&mut self, key: &[u8]) {
        let h1 = fnv1a_64(key);
        let h2 = fnv1b_64(key);
        for i in 0..K_HASHES {
            let pos = h1.wrapping_add((i as u64).wrapping_mul(h2)) as usize % self.num_bits;
            self.set_bit(pos);
        }
    }

    fn contains(&self, key: &[u8]) -> bool {
        let h1 = fnv1a_64(key);
        let h2 = fnv1b_64(key);
        for i in 0..K_HASHES {
            let pos = h1.wrapping_add((i as u64).wrapping_mul(h2)) as usize % self.num_bits;
            if !self.get_bit(pos) {
                return false;
            }
        }
        true
    }
}

/// Collection of per-demographic-slice Bloom filters.
pub struct DemographicBloom {
    filters: HashMap<DemographicKey, BloomFilter>,
    /// Expected subjects per slice (used for initial filter sizing).
    slice_capacity: usize,
}

impl DemographicBloom {
    pub fn new(slice_capacity: usize) -> Self {
        Self {
            filters: HashMap::new(),
            slice_capacity,
        }
    }

    /// Register a subject under the given demographic slice.
    pub fn insert(&mut self, key: &DemographicKey, subject_id: &str) {
        let cap = self.slice_capacity;
        let filter = self
            .filters
            .entry(key.clone())
            .or_insert_with(|| BloomFilter::new(cap));
        filter.insert(subject_id.as_bytes());
    }

    /// Test whether a subject_id might be in the given slice (may return false positive).
    pub fn may_contain(&self, key: &DemographicKey, subject_id: &str) -> bool {
        match self.filters.get(key) {
            None => false,
            Some(f) => f.contains(subject_id.as_bytes()),
        }
    }

    /// Return the keys of all slices that likely contain `subject_id`.
    pub fn candidate_slices(&self, subject_id: &str) -> Vec<&DemographicKey> {
        self.filters
            .iter()
            .filter_map(|(k, f)| {
                if f.contains(subject_id.as_bytes()) {
                    Some(k)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn slice_count(&self) -> usize {
        self.filters.len()
    }
}
