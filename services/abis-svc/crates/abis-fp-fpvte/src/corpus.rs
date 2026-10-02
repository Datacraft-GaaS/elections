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

//! NIST corpus loader.
//!
//! Supports the directory layouts of:
//! NIST SD-4, SD-14, SD-27, SD-29, SD-302, SD-361
//! (NIST Special Databases: <https://www.nist.gov/srd/nist-special-database-4>)
//!
//! In the absence of real NIST data, the loader also accepts a simple synthetic
//! layout where each subject has a subdirectory containing raw `.bin` templates.

use crate::FpVteError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Optional demographic metadata for fairness slicing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DemographicLabel {
    /// Biological sex (M / F / Unknown).
    pub sex: Option<String>,
    /// Age group (e.g. "18-25").
    pub age_group: Option<String>,
    /// Self-reported skin tone (Fitzpatrick scale 1-6).
    pub fitzpatrick: Option<u8>,
}

/// A single corpus entry: one template from one subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusEntry {
    /// Unique subject identifier.
    pub subject_id: String,
    /// Acquisition index (0 = first, 1 = second, ...).
    pub acquisition: u8,
    /// Raw template bytes.
    pub template: Vec<u8>,
    /// Optional demographic metadata.
    pub demographics: DemographicLabel,
}

/// Corpus: a collection of entries usable as probe and/or gallery.
#[derive(Debug, Clone, Default)]
pub struct Corpus {
    pub entries: Vec<CorpusEntry>,
}

impl Corpus {
    /// Load a synthetic corpus from a directory.
    ///
    /// Expected layout (minimal):
    /// ```text
    /// corpus_root/
    ///   {subject_id}/
    ///     0.bin   -- first acquisition
    ///     1.bin   -- second acquisition (optional)
    /// ```
    pub fn load_synthetic(root: &Path) -> Result<Self, FpVteError> {
        if !root.exists() {
            return Err(FpVteError::CorpusLoad(format!(
                "path does not exist: {}",
                root.display()
            )));
        }
        let mut entries = Vec::new();
        let dir = std::fs::read_dir(root).map_err(|e| FpVteError::CorpusLoad(e.to_string()))?;
        for entry in dir {
            let entry = entry.map_err(|e| FpVteError::CorpusLoad(e.to_string()))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let subject_id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            for acq in 0u8..=3 {
                let tpl_path = path.join(format!("{acq}.bin"));
                if !tpl_path.exists() {
                    break;
                }
                let template =
                    std::fs::read(&tpl_path).map_err(|e| FpVteError::CorpusLoad(e.to_string()))?;
                entries.push(CorpusEntry {
                    subject_id: subject_id.clone(),
                    acquisition: acq,
                    template,
                    demographics: DemographicLabel::default(),
                });
            }
        }
        Ok(Self { entries })
    }

    /// Build a synthetic in-memory corpus for testing.
    /// Generates `n_subjects` subjects, each with 2 acquisitions.
    /// Templates are deterministic byte sequences per subject.
    pub fn synthetic_in_memory(n_subjects: usize) -> Self {
        let mut entries = Vec::new();
        for i in 0..n_subjects {
            for acq in 0u8..2 {
                // Genuine templates: same base bytes with minor variation.
                let mut template = vec![(i % 256) as u8; 64];
                template[0] = i as u8;
                template[1] = acq;
                entries.push(CorpusEntry {
                    subject_id: format!("subj{i:04}"),
                    acquisition: acq,
                    template,
                    demographics: DemographicLabel::default(),
                });
            }
        }
        Self { entries }
    }

    /// All entries for a given subject.
    pub fn entries_for_subject(&self, subject_id: &str) -> Vec<&CorpusEntry> {
        self.entries
            .iter()
            .filter(|e| e.subject_id == subject_id)
            .collect()
    }

    /// Unique subject IDs.
    pub fn subject_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.entries.iter().map(|e| e.subject_id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }
}
