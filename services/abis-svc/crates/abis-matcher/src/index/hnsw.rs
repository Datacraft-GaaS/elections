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

//! Minimal HNSW (Hierarchical Navigable Small World) index for ANN search.
//!
//! Parameters: M=32 (max connections per layer), ef_construction=200, ef_search=100.
//! Suitable for 512-d face embeddings at 30M scale when sharded.
//!
//! Reference: Malkov & Yashunin, "Efficient and robust approximate nearest neighbor search
//! using Hierarchical Navigable Small World graphs", IEEE TPAMI 2020.

use std::collections::{BinaryHeap, HashMap, HashSet};

use super::{Candidate, IndexError, TemplateIndex};

const M: usize = 32;
const M_MAX0: usize = 64; // 2*M for layer 0
const EF_CONSTRUCTION: usize = 200;
pub const EF_SEARCH: usize = 100;

#[derive(Debug, Clone)]
struct Node {
    /// Subject identifier.
    id: String,
    /// Embedding vector.
    data: Vec<f32>,
    /// Layer index this node was assigned (0 = base).
    max_layer: usize,
    /// neighbors[layer] = sorted neighbor node indices.
    neighbors: Vec<Vec<usize>>,
}

/// Ordered pair for min-heap (distance, index).
#[derive(Clone, PartialEq)]
struct DistIdx {
    dist: f32,
    idx: usize,
}

impl PartialOrd for DistIdx {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for DistIdx {}

impl Ord for DistIdx {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Max-heap by default; negate for min-heap.
        other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn squared_l2(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum()
}

/// HNSW graph index.
///
/// Mutation (`add`, `remove`) requires `&mut self`; `search` is read-only.
pub struct HnswIndex {
    inner: HnswInner,
    dim: usize,
}

struct HnswInner {
    nodes: Vec<Node>,
    id_map: HashMap<String, usize>,
    entry_point: Option<usize>,
    max_level: usize,
    /// Multiplicative factor for level generation: 1 / ln(M).
    ml: f64,
}

impl HnswInner {
    fn new() -> Self {
        let ml = 1.0 / (M as f64).ln();
        Self {
            nodes: Vec::new(),
            id_map: HashMap::new(),
            entry_point: None,
            max_level: 0,
            ml,
        }
    }

    /// Draw a random level for a new node (exponential distribution).
    fn random_level(&self, node_idx: usize) -> usize {
        // Deterministic pseudo-random using node index (avoids rand dependency).
        let mut h = node_idx as u64 ^ 0xDEAD_BEEF_CAFE_1337;
        h ^= h >> 33;
        h = h.wrapping_mul(0xFF51AFD7ED558CCD);
        h ^= h >> 33;
        h = h.wrapping_mul(0xC4CEB9FE1A85EC53);
        h ^= h >> 33;

        let mut level = 0usize;
        let mut r = (h & 0xFFFF) as f64 / 65536.0;
        while r < self.ml && level < 16 {
            level += 1;
            r /= self.ml;
        }
        level
    }

    /// Greedy beam search on a single layer. Returns `ef` nearest candidates to `q`.
    fn search_layer(&self, q: &[f32], entry: usize, ef: usize, level: usize) -> Vec<usize> {
        let mut visited: HashSet<usize> = HashSet::new();
        // candidates: min-heap by distance.
        let mut candidates: BinaryHeap<DistIdx> = BinaryHeap::new();
        // dynamic list W: max-heap by distance (keep ef nearest).
        let mut w: BinaryHeap<std::cmp::Reverse<DistIdx>> = BinaryHeap::new();

        let d_entry = squared_l2(q, &self.nodes[entry].data);
        visited.insert(entry);
        candidates.push(DistIdx {
            dist: d_entry,
            idx: entry,
        });
        w.push(std::cmp::Reverse(DistIdx {
            dist: d_entry,
            idx: entry,
        }));

        while let Some(DistIdx {
            dist: c_dist,
            idx: c_idx,
        }) = candidates.pop()
        {
            // f = worst in W.
            let f_dist = w.peek().map(|r| r.0.dist).unwrap_or(f32::MAX);
            if c_dist > f_dist && w.len() >= ef {
                break;
            }
            let node = &self.nodes[c_idx];
            let neighbors = if level < node.neighbors.len() {
                &node.neighbors[level]
            } else {
                continue;
            };
            for &e in neighbors {
                if visited.insert(e) {
                    let d_e = squared_l2(q, &self.nodes[e].data);
                    let f_worst = w.peek().map(|r| r.0.dist).unwrap_or(f32::MAX);
                    if d_e < f_worst || w.len() < ef {
                        candidates.push(DistIdx { dist: d_e, idx: e });
                        w.push(std::cmp::Reverse(DistIdx { dist: d_e, idx: e }));
                        if w.len() > ef {
                            w.pop();
                        }
                    }
                }
            }
        }
        w.into_iter().map(|r| r.0.idx).collect()
    }

    /// Select `m` neighbors using simple greedy selection (nearest first).
    fn select_neighbors(&self, q: &[f32], candidates: Vec<usize>, m: usize) -> Vec<usize> {
        let mut pairs: Vec<(f32, usize)> = candidates
            .into_iter()
            .map(|idx| (squared_l2(q, &self.nodes[idx].data), idx))
            .collect();
        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        pairs.into_iter().take(m).map(|(_, idx)| idx).collect()
    }

    fn insert_node(&mut self, id: String, data: Vec<f32>) -> usize {
        let node_idx = self.nodes.len();
        let level = self.random_level(node_idx);

        let neighbors = vec![Vec::new(); level + 1];
        self.nodes.push(Node {
            id: id.clone(),
            data: data.clone(),
            max_layer: level,
            neighbors,
        });
        self.id_map.insert(id, node_idx);

        if self.entry_point.is_none() {
            self.entry_point = Some(node_idx);
            self.max_level = level;
            return node_idx;
        }

        // entry_point is Some because we checked is_none() above and returned early.
        let ep = match self.entry_point {
            Some(ep) => ep,
            None => return node_idx,
        };
        let mut curr_ep = ep;
        let top = self.max_level;

        // Descend from top to level+1 (greedy, ef=1).
        for lc in (level + 1..=top).rev() {
            let candidates = self.search_layer(&data, curr_ep, 1, lc);
            if let Some(&nearest) = candidates.first() {
                curr_ep = nearest;
            }
        }

        // Insert into layers 0..=level.
        for lc in (0..=level.min(top)).rev() {
            let m_lc = if lc == 0 { M_MAX0 } else { M };
            let candidates = self.search_layer(&data, curr_ep, EF_CONSTRUCTION, lc);
            if let Some(&nearest) = candidates.first() {
                curr_ep = nearest;
            }
            let neighbors = self.select_neighbors(&data, candidates, m_lc);

            // Add bidirectional edges.
            self.nodes[node_idx].neighbors[lc] = neighbors.clone();
            for nb in &neighbors {
                let nb = *nb;
                let max_nb = if lc == 0 { M_MAX0 } else { M };
                if !self.nodes[nb].neighbors[lc].contains(&node_idx) {
                    self.nodes[nb].neighbors[lc].push(node_idx);
                    // Prune if over capacity.
                    if self.nodes[nb].neighbors[lc].len() > max_nb {
                        let nb_data = self.nodes[nb].data.clone();
                        let current: Vec<usize> = self.nodes[nb].neighbors[lc].clone();
                        let pruned = self.select_neighbors(&nb_data, current, max_nb);
                        self.nodes[nb].neighbors[lc] = pruned;
                    }
                }
            }
        }

        if level > top {
            self.entry_point = Some(node_idx);
            self.max_level = level;
        }
        node_idx
    }

    fn knn_search(&self, q: &[f32], k: usize) -> Vec<Candidate> {
        let ep = match self.entry_point {
            None => return vec![],
            Some(ep) => ep,
        };

        let mut curr_ep = ep;
        let top = self.max_level;

        // Descend to layer 1 greedily.
        for lc in (1..=top).rev() {
            let candidates = self.search_layer(q, curr_ep, 1, lc);
            if let Some(&nearest) = candidates.first() {
                curr_ep = nearest;
            }
        }

        // Beam search at layer 0.
        let candidates = self.search_layer(q, curr_ep, EF_SEARCH.max(k), 0);
        let mut pairs: Vec<(f32, usize)> = candidates
            .into_iter()
            .map(|idx| (squared_l2(q, &self.nodes[idx].data), idx))
            .collect();
        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        pairs
            .into_iter()
            .take(k)
            .map(|(dist, idx)| Candidate {
                subject_id: self.nodes[idx].id.clone(),
                distance: dist.sqrt(),
            })
            .collect()
    }
}

impl HnswIndex {
    pub fn new(dim: usize) -> Self {
        Self {
            inner: HnswInner::new(),
            dim,
        }
    }
}

impl TemplateIndex for HnswIndex {
    fn add(&mut self, subject_id: &str, embedding: &[f32]) -> Result<(), IndexError> {
        if embedding.len() != self.dim {
            return Err(IndexError::DimMismatch {
                expected: self.dim,
                got: embedding.len(),
            });
        }
        self.inner
            .insert_node(subject_id.to_string(), embedding.to_vec());
        Ok(())
    }

    fn remove(&mut self, subject_id: &str) -> Result<(), IndexError> {
        let idx = self
            .inner
            .id_map
            .remove(subject_id)
            .ok_or_else(|| IndexError::NotFound(subject_id.to_string()))?;
        self.inner.nodes[idx]
            .neighbors
            .iter_mut()
            .for_each(|l| l.clear());
        Ok(())
    }

    fn search(&self, probe: &[f32], k: usize) -> Result<Vec<Candidate>, IndexError> {
        if probe.len() != self.dim {
            return Err(IndexError::DimMismatch {
                expected: self.dim,
                got: probe.len(),
            });
        }
        Ok(self.inner.knn_search(probe, k))
    }

    fn len(&self) -> usize {
        self.inner.id_map.len()
    }
}
