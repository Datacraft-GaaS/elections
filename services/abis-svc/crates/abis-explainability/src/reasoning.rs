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

use crate::ExplainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityResult {
    pub modality: String,
    pub matched_minutiae: u32,
    pub probe_total: u32,
    pub gallery_total: u32,
    pub fusion_score: f32,
    pub threshold: f32,
}

pub fn generate_reasoning_text(
    results: &[ModalityResult],
    overall_decision: bool,
) -> Result<String, ExplainError> {
    if results.is_empty() {
        return Err(ExplainError::EmptyTemplate);
    }

    let mut lines = Vec::new();
    for r in results {
        let decision = if r.fusion_score >= r.threshold {
            "MATCH"
        } else {
            "NO MATCH"
        };
        let verb = if r.fusion_score >= r.threshold {
            "exceeds"
        } else {
            "is below"
        };
        lines.push(format!(
            "{}: {} minutiae matched of {} in probe; {} of {} in gallery. Fusion score {:.2} {} threshold {:.2}. {}.",
            r.modality,
            r.matched_minutiae,
            r.probe_total,
            r.matched_minutiae,
            r.gallery_total,
            r.fusion_score,
            verb,
            r.threshold,
            decision,
        ));
    }

    let overall = if overall_decision {
        "MATCH"
    } else {
        "NO MATCH"
    };
    lines.push(format!("Overall decision: {}.", overall));

    Ok(lines.join("\n"))
}
