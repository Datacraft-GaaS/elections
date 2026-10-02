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

//! Deterministic hash-based mock matcher for testing.

use sha2::{Digest, Sha256};

use crate::domain::{MatchOutcome, MatchResult, ModalityScore, TemplateIndex};
use crate::pipeline::MatchPipeline;

pub struct MockMatcher;

fn sha256_bytes(s: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hasher.finalize().into()
}

/// Score for 1:1 match using multi-byte XOR normalization.
/// Same string -> score 1.0. Different strings -> average XOR byte / 255 ≈ 0.5.
fn xor_similarity(probe_ref: &str, gallery_ref: &str) -> f32 {
    if probe_ref == gallery_ref {
        return 1.0;
    }
    let p = sha256_bytes(probe_ref);
    let g = sha256_bytes(gallery_ref);
    // XOR byte average: 0 = identical, 127/255 ≈ 0.5 for random pairs.
    // Convert to similarity: 1 - (avg_xor / 255).
    let avg_xor: f32 = p
        .iter()
        .zip(g.iter())
        .map(|(a, b)| (a ^ b) as f32)
        .sum::<f32>()
        / 32.0;
    1.0 - (avg_xor / 255.0)
}

fn outcome(score: f32, threshold: f32) -> MatchOutcome {
    if score >= threshold {
        MatchOutcome::Match
    } else {
        MatchOutcome::NoMatch
    }
}

fn new_result_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

impl MatchPipeline for MockMatcher {
    fn match_1n(&self, probe_ref: &str, index: &[TemplateIndex], threshold: f32) -> MatchResult {
        let probe_bytes = sha256_bytes(probe_ref);
        let best_score = index.iter().fold(0.0f32, |best, entry| {
            let gallery_bytes = sha256_bytes(&entry.template_hash);
            let avg_xor: f32 = probe_bytes
                .iter()
                .zip(gallery_bytes.iter())
                .map(|(a, b)| (a ^ b) as f32)
                .sum::<f32>()
                / 32.0;
            let similarity = 1.0 - (avg_xor / 255.0);
            if similarity > best {
                similarity
            } else {
                best
            }
        });
        let confidence = if index.is_empty() { 0.0 } else { best_score };
        MatchResult {
            result_id: new_result_id(),
            request_id: probe_ref.to_string(),
            outcome: outcome(confidence, threshold),
            confidence_score: confidence,
            modality_scores: vec![ModalityScore {
                modality: "fingerprint".to_string(),
                score: confidence,
            }],
        }
    }

    fn match_11(&self, probe_ref: &str, gallery_ref: &str, threshold: f32) -> MatchResult {
        let score = xor_similarity(probe_ref, gallery_ref);
        MatchResult {
            result_id: new_result_id(),
            request_id: probe_ref.to_string(),
            outcome: outcome(score, threshold),
            confidence_score: score,
            modality_scores: vec![ModalityScore {
                modality: "fingerprint".to_string(),
                score,
            }],
        }
    }
}
