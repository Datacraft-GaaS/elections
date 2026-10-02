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

//! Connected-components algorithm over the match graph using union-find.

use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{ClusterRecord, MatchEdge};
use crate::error::ClusterError;

struct UnionFind {
    parent: HashMap<Uuid, Uuid>,
    rank: HashMap<Uuid, u32>,
}

impl UnionFind {
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }

    fn make_set(&mut self, x: Uuid) {
        self.parent.entry(x).or_insert(x);
        self.rank.entry(x).or_insert(0);
    }

    fn find(&mut self, x: Uuid) -> Uuid {
        let parent = *self.parent.get(&x).unwrap_or(&x);
        if parent == x {
            return x;
        }
        let root = self.find(parent);
        self.parent.insert(x, root);
        root
    }

    fn union(&mut self, a: Uuid, b: Uuid) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        let rank_a = *self.rank.get(&ra).unwrap_or(&0);
        let rank_b = *self.rank.get(&rb).unwrap_or(&0);
        if rank_a < rank_b {
            self.parent.insert(ra, rb);
        } else if rank_a > rank_b {
            self.parent.insert(rb, ra);
        } else {
            self.parent.insert(rb, ra);
            let r = self.rank.entry(ra).or_insert(0);
            *r += 1;
        }
    }
}

/// Compute connected components from match edges. Returns one ClusterRecord per component >= 2 members.
pub fn connected_components(edges: &[MatchEdge]) -> Result<Vec<ClusterRecord>, ClusterError> {
    if edges.is_empty() {
        return Ok(vec![]);
    }
    let mut uf = UnionFind::new();
    for e in edges {
        uf.make_set(e.subject_a);
        uf.make_set(e.subject_b);
        uf.union(e.subject_a, e.subject_b);
    }

    let all_subjects: Vec<Uuid> = {
        let mut s: Vec<Uuid> = uf.parent.keys().copied().collect();
        s.sort();
        s.dedup();
        s
    };
    let mut components: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for id in all_subjects {
        let root = uf.find(id);
        components.entry(root).or_default().push(id);
    }

    let mut edge_counts: HashMap<Uuid, usize> = HashMap::new();
    for e in edges {
        let root = uf.find(e.subject_a);
        *edge_counts.entry(root).or_insert(0) += 1;
    }

    let mut clusters: Vec<ClusterRecord> = components
        .into_iter()
        .filter(|(_, members)| members.len() >= 2)
        .map(|(root, members)| {
            let ec = *edge_counts.get(&root).unwrap_or(&0);
            ClusterRecord::new(members, ec)
        })
        .collect();

    clusters.sort_by_key(|c| c.member_ids.len());
    Ok(clusters)
}
