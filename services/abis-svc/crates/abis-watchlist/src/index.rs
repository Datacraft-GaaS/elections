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

use std::collections::HashSet;

// Bloom filter using double hashing (Kirsch-Mitzenmacher trick).
pub struct BloomFilter {
    bits: Vec<u64>,
    k: usize,
    m: usize,
}

impl BloomFilter {
    pub fn new(capacity: usize, false_positive_rate: f64) -> Self {
        // m = -n * ln(p) / (ln(2)^2)
        let ln2 = std::f64::consts::LN_2;
        let m = (-(capacity as f64) * false_positive_rate.ln() / (ln2 * ln2)).ceil() as usize;
        let m = m.max(64);
        let k = ((m as f64 / capacity as f64) * ln2).ceil() as usize;
        let k = k.max(1);
        let words = (m + 63) / 64;
        Self {
            bits: vec![0u64; words],
            k,
            m,
        }
    }

    fn hash1(key: &str, seed: u64) -> u64 {
        // FNV-1a variant seeded.
        let mut h: u64 = 14695981039346656037u64 ^ seed.wrapping_mul(1099511628211);
        for b in key.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(1099511628211);
        }
        h
    }

    fn hash2(key: &str, seed: u64) -> u64 {
        // djb2 variant seeded.
        let mut h: u64 = 5381u64.wrapping_add(seed.wrapping_mul(33));
        for b in key.bytes() {
            h = h.wrapping_mul(33).wrapping_add(b as u64);
        }
        h
    }

    fn bit_indices(&self, key: &str) -> impl Iterator<Item = usize> + '_ {
        let h1 = Self::hash1(key, 0);
        let h2 = Self::hash2(key, 1);
        let m = self.m;
        (0..self.k).map(move |i| {
            let combined = h1.wrapping_add((i as u64).wrapping_mul(h2));
            (combined % m as u64) as usize
        })
    }

    pub fn insert(&mut self, key: &str) {
        let indices: Vec<usize> = self.bit_indices(key).collect();
        for idx in indices {
            self.bits[idx / 64] |= 1u64 << (idx % 64);
        }
    }

    pub fn may_contain(&self, key: &str) -> bool {
        self.bit_indices(key)
            .all(|idx| (self.bits[idx / 64] >> (idx % 64)) & 1 == 1)
    }
}

pub struct WatchlistIndex {
    pub watchlist_id: String,
    pub modality: String,
    hashes: HashSet<String>,
    bloom: BloomFilter,
}

impl WatchlistIndex {
    pub fn new(watchlist_id: String, modality: String, capacity: usize) -> Self {
        let cap = capacity.max(16);
        Self {
            watchlist_id,
            modality,
            hashes: HashSet::with_capacity(cap),
            bloom: BloomFilter::new(cap, 0.01),
        }
    }

    pub fn insert(&mut self, template_hash: String) {
        self.bloom.insert(&template_hash);
        self.hashes.insert(template_hash);
    }

    pub fn remove(&mut self, template_hash: &str) -> bool {
        // Note: Bloom filter is not updated on remove (standard limitation).
        self.hashes.remove(template_hash)
    }

    pub fn contains(&self, template_hash: &str) -> bool {
        if !self.bloom.may_contain(template_hash) {
            return false;
        }
        self.hashes.contains(template_hash)
    }

    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }
}

pub struct MultiIndex {
    indices: Vec<WatchlistIndex>,
}

impl MultiIndex {
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    pub fn add_index(&mut self, index: WatchlistIndex) {
        self.indices.push(index);
    }

    pub fn contains_any(&self, template_hash: &str) -> Option<&WatchlistIndex> {
        self.indices.iter().find(|idx| idx.contains(template_hash))
    }

    pub fn remove_watchlist(&mut self, watchlist_id: &str) {
        self.indices.retain(|idx| idx.watchlist_id != watchlist_id);
    }

    pub fn get_index(&self, watchlist_id: &str, modality: &str) -> Option<&WatchlistIndex> {
        self.indices
            .iter()
            .find(|idx| idx.watchlist_id == watchlist_id && idx.modality == modality)
    }

    pub fn get_index_mut(
        &mut self,
        watchlist_id: &str,
        modality: &str,
    ) -> Option<&mut WatchlistIndex> {
        self.indices
            .iter_mut()
            .find(|idx| idx.watchlist_id == watchlist_id && idx.modality == modality)
    }

    pub fn indices_for_modality<'a>(
        &'a self,
        modality: &'a str,
    ) -> impl Iterator<Item = &'a WatchlistIndex> {
        self.indices
            .iter()
            .filter(move |idx| idx.modality == modality)
    }
}

impl Default for MultiIndex {
    fn default() -> Self {
        Self::new()
    }
}
