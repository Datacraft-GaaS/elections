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

//! IREX corpus: test-set loading and pair generation.

use eel_abis_iris_encode::IrisTemplate;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// A single entry in an IREX evaluation corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusEntry {
    /// Unique acquisition identifier (e.g. "subject_1_a").
    pub id: String,
    /// Processed iris template.
    pub template: IrisTemplate,
    /// Demographic or sensor group label.
    pub group: String,
    /// Overall quality score in [0, 1].
    pub quality: f32,
}

/// A validated, indexed IREX corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrexCorpus {
    pub entries: Vec<CorpusEntry>,
}

/// Errors returned when constructing an [`IrexCorpus`].
#[derive(Debug, Error)]
pub enum CorpusError {
    #[error("empty corpus")]
    Empty,
    #[error("duplicate id: {0}")]
    DuplicateId(String),
}

/// Extract the subject prefix from an acquisition ID.
///
/// "subject_1_a" -> "subject_1", "subject_1" -> "subject_1".
fn id_prefix(id: &str) -> &str {
    // Find the last underscore; everything before it is the subject prefix.
    // If no underscore exists the whole id is the prefix.
    match id.rfind('_') {
        Some(pos) => &id[..pos],
        None => id,
    }
}

impl IrexCorpus {
    /// Build a validated corpus from a list of entries.
    ///
    /// Returns an error if the list is empty or contains duplicate IDs.
    pub fn new(entries: Vec<CorpusEntry>) -> Result<Self, CorpusError> {
        if entries.is_empty() {
            return Err(CorpusError::Empty);
        }
        let mut seen = HashSet::new();
        for e in &entries {
            if !seen.insert(e.id.clone()) {
                return Err(CorpusError::DuplicateId(e.id.clone()));
            }
        }
        Ok(Self { entries })
    }

    /// Number of entries in the corpus.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true when the corpus has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return all entries belonging to `group`.
    pub fn by_group<'a>(&'a self, group: &str) -> Vec<&'a CorpusEntry> {
        self.entries.iter().filter(|e| e.group == group).collect()
    }

    /// Return all genuine pairs: two acquisitions of the same subject within the same group.
    ///
    /// Pairs share the same ID prefix (everything before the last `_`).
    pub fn genuine_pairs(&self) -> Vec<(&CorpusEntry, &CorpusEntry)> {
        let mut pairs = Vec::new();
        let n = self.entries.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let a = &self.entries[i];
                let b = &self.entries[j];
                if a.group == b.group && id_prefix(&a.id) == id_prefix(&b.id) {
                    pairs.push((a, b));
                }
            }
        }
        pairs
    }

    /// Return a capped sample of impostor pairs: different subject, same group.
    ///
    /// Capped at 1000 pairs to keep evaluations fast.
    pub fn impostor_pairs(&self) -> Vec<(&CorpusEntry, &CorpusEntry)> {
        let mut pairs = Vec::new();
        let n = self.entries.len();
        'outer: for i in 0..n {
            for j in (i + 1)..n {
                let a = &self.entries[i];
                let b = &self.entries[j];
                if a.group == b.group && id_prefix(&a.id) != id_prefix(&b.id) {
                    pairs.push((a, b));
                    if pairs.len() >= 1000 {
                        break 'outer;
                    }
                }
            }
        }
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_encode::{
        daugman_code::DaugmanCode, pie::PieCode, IrisTemplate, IRIS_CODE_BYTES,
    };

    fn synthetic_template(seed: u8) -> IrisTemplate {
        let code: [u8; IRIS_CODE_BYTES] = std::array::from_fn(|_| seed);
        let mask: [u8; IRIS_CODE_BYTES] = [0xFF; IRIS_CODE_BYTES];
        IrisTemplate {
            daugman: DaugmanCode { code, mask },
            pie: PieCode { code, mask },
            quality: seed as f32 / 255.0,
        }
    }

    fn make_entry(id: &str, group: &str, seed: u8) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            template: synthetic_template(seed),
            group: group.to_string(),
            quality: seed as f32 / 255.0,
        }
    }

    #[test]
    fn corpus_rejects_empty() {
        assert!(matches!(IrexCorpus::new(vec![]), Err(CorpusError::Empty)));
    }

    #[test]
    fn corpus_rejects_duplicate_ids() {
        let entries = vec![
            make_entry("sub_1_a", "g1", 10),
            make_entry("sub_1_a", "g1", 20),
        ];
        assert!(matches!(
            IrexCorpus::new(entries),
            Err(CorpusError::DuplicateId(_))
        ));
    }

    #[test]
    fn corpus_accepts_valid_entries() {
        let entries = vec![
            make_entry("sub_1_a", "g1", 10),
            make_entry("sub_1_b", "g1", 20),
        ];
        let corpus = IrexCorpus::new(entries).expect("valid corpus");
        assert_eq!(corpus.len(), 2);
        assert!(!corpus.is_empty());
    }

    #[test]
    fn by_group_filters_correctly() {
        let entries = vec![
            make_entry("sub_1_a", "g1", 10),
            make_entry("sub_2_a", "g2", 20),
            make_entry("sub_3_a", "g1", 30),
        ];
        let corpus = IrexCorpus::new(entries).unwrap();
        let g1 = corpus.by_group("g1");
        assert_eq!(g1.len(), 2);
        assert!(g1.iter().all(|e| e.group == "g1"));
    }

    #[test]
    fn genuine_pairs_same_prefix_only() {
        let entries = vec![
            make_entry("sub_1_a", "g1", 10),
            make_entry("sub_1_b", "g1", 20),
            make_entry("sub_2_a", "g1", 30),
        ];
        let corpus = IrexCorpus::new(entries).unwrap();
        let pairs = corpus.genuine_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(id_prefix(&pairs[0].0.id), "sub_1");
        assert_eq!(id_prefix(&pairs[0].1.id), "sub_1");
    }

    #[test]
    fn impostor_pairs_different_prefix_only() {
        let entries = vec![
            make_entry("sub_1_a", "g1", 10),
            make_entry("sub_1_b", "g1", 20),
            make_entry("sub_2_a", "g1", 30),
        ];
        let corpus = IrexCorpus::new(entries).unwrap();
        let pairs = corpus.impostor_pairs();
        for (a, b) in &pairs {
            assert_ne!(id_prefix(&a.id), id_prefix(&b.id));
        }
        assert!(!pairs.is_empty());
    }

    #[test]
    fn impostor_pairs_capped_at_1000() {
        // Build enough entries to exceed the cap.
        let mut entries = Vec::new();
        for s in 0..60u8 {
            entries.push(make_entry(&format!("sub_{s}_a"), "g1", s));
        }
        let corpus = IrexCorpus::new(entries).unwrap();
        assert!(corpus.impostor_pairs().len() <= 1000);
    }
}
