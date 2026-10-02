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

//! CPU fallback matcher using iterator-based parallelism over the shard tier.

use crate::domain::{AcceleratorCapabilities, AcceleratorKind, MatchQuery, Score};
use crate::error::AcceleratorError;

pub struct CpuFallbackMatcher;

impl CpuFallbackMatcher {
    pub fn batch_1n(&self, query: &MatchQuery) -> Result<Vec<Score>, AcceleratorError> {
        if query.gallery_ids.len() != query.gallery_templates.len() {
            return Err(AcceleratorError::TemplateSizeMismatch);
        }
        let probe_len = query.probe_template.len();
        let scores = query
            .gallery_ids
            .iter()
            .zip(query.gallery_templates.iter())
            .map(|(&gid, gallery)| {
                let overlap = if probe_len == 0 || gallery.is_empty() {
                    0.0
                } else {
                    let min_len = probe_len.min(gallery.len());
                    let dots: f64 = query.probe_template[..min_len]
                        .iter()
                        .zip(gallery[..min_len].iter())
                        .map(|(&a, &b)| a as f64 * b as f64)
                        .sum();
                    (dots / (probe_len as f64 * 255.0 * min_len as f64)).min(1.0)
                };
                Score {
                    probe_id: query.probe_id,
                    gallery_id: gid,
                    score: overlap,
                }
            })
            .collect();
        Ok(scores)
    }

    pub fn capabilities(&self) -> AcceleratorCapabilities {
        AcceleratorCapabilities {
            kind: AcceleratorKind::Cpu,
            max_batch_size: usize::MAX,
            throughput_kps: 10.0,
            power_w: 65.0,
            healthy: true,
        }
    }
}
