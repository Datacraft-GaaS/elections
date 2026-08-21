//! Split-view detection: alert if two witnesses signed inconsistent STHs at the same tree_size.

use std::collections::HashMap;

use tlog_domain::SignedTreeHead;

pub struct SplitViewDetector {
    /// Maps tree_size -> list of (witness_id, root_hash).
    seen: HashMap<u64, Vec<(String, String)>>,
}

impl SplitViewDetector {
    pub fn new() -> Self {
        Self {
            seen: HashMap::new(),
        }
    }

    /// Record a cosignature observation. Returns true if a split view is detected.
    pub fn observe(&mut self, witness_id: &str, sth: &SignedTreeHead) -> bool {
        let entry = self.seen.entry(sth.tree_size).or_default();
        let root = sth.root_hash.clone();
        let mut split = false;
        for (_, existing_root) in entry.iter() {
            if existing_root != &root {
                split = true;
                tracing::error!(
                    tree_size = sth.tree_size,
                    witness_a = %witness_id,
                    root_a = %root,
                    root_b = %existing_root,
                    "SPLIT VIEW DETECTED"
                );
            }
        }
        entry.push((witness_id.to_string(), root));
        split
    }
}

impl Default for SplitViewDetector {
    fn default() -> Self {
        Self::new()
    }
}
