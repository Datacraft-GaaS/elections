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

//! Split-view detection: alert if two witnesses signed inconsistent STHs at the same tree_size.

use std::collections::HashMap;

use tlog_domain::SignedTreeHead;

pub struct SplitViewDetector {
    /// Maps tree_size -> list of (witness_id, root_hash).
    seen: HashMap<u64, Vec<(String, String)>>,
    /// All split-view events ever observed by this detector.
    pub findings: Vec<Finding>,
}

impl SplitViewDetector {
    pub fn new() -> Self {
        Self {
            seen: HashMap::new(),
            findings: Vec::new(),
        }
    }

    /// Record a cosignature observation. Returns true if a split view is detected.
    pub fn observe(&mut self, witness_id: &str, sth: &SignedTreeHead) -> bool {
        self.observe_root(witness_id, sth.tree_size, &sth.root_hash)
    }

    /// Primitive observation: same detection without requiring a full
    /// `SignedTreeHead` (used by gossip polling over HTTP).
    pub fn observe_root(&mut self, witness_id: &str, tree_size: u64, root_hash: &str) -> bool {
        let entry = self.seen.entry(tree_size).or_default();
        let root = root_hash.to_string();
        let mut split = false;
        for (_, existing_root) in entry.iter() {
            if existing_root != &root {
                split = true;
                tracing::error!(
                    tree_size = tree_size,
                    witness_a = %witness_id,
                    root_a = %root,
                    root_b = %existing_root,
                    "SPLIT VIEW DETECTED"
                );
            }
        }
        entry.push((witness_id.to_string(), root.clone()));
        if split {
            self.findings.push(Finding {
                tree_size,
                reported_by: witness_id.to_string(),
                conflicting_root: root,
                detected_at: chrono::Utc::now(),
            });
        }
        split
    }
}

/// One detected split-view event, retained for operator retrieval.
#[derive(Debug, Clone)]
pub struct Finding {
    pub tree_size: u64,
    pub reported_by: String,
    pub conflicting_root: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

impl Default for SplitViewDetector {
    fn default() -> Self {
        Self::new()
    }
}
