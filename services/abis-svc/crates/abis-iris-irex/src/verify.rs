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

//! IREX 1:1 verification track.

use crate::corpus::CorpusEntry;
use serde::{Deserialize, Serialize};

/// Result of a 1:1 verification comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub probe_id: String,
    pub gallery_id: String,
    /// Similarity score in [0, 1] (1 - Hamming distance).
    pub score: f32,
    /// True when score >= threshold.
    pub decision: bool,
    pub threshold: f32,
}

/// 1:1 verifier calibrated to a fixed acceptance threshold.
#[derive(Debug, Clone)]
pub struct IrexVerifier {
    pub threshold: f32,
    pub min_overlap: f32,
}

impl IrexVerifier {
    /// Create a verifier with the given threshold.
    ///
    /// `min_overlap` defaults to 0.3 (30% of bits must be unmasked).
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            min_overlap: 0.3,
        }
    }

    /// Compare probe and gallery templates and return a [`VerifyResult`].
    ///
    /// Score = 1.0 - Hamming distance. Returns 0.0 when the overlap is
    /// insufficient (Hamming distance returns None).
    pub fn verify(&self, probe: &CorpusEntry, gallery: &CorpusEntry) -> VerifyResult {
        let hd = probe
            .template
            .hamming_distance(&gallery.template, self.min_overlap);
        let score = hd.map_or(0.0, |d| 1.0 - d);
        VerifyResult {
            probe_id: probe.id.clone(),
            gallery_id: gallery.id.clone(),
            score,
            decision: score >= self.threshold,
            threshold: self.threshold,
        }
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

    fn make_entry(id: &str, seed: u8) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            template: synthetic_template(seed),
            group: "g1".to_string(),
            quality: seed as f32 / 255.0,
        }
    }

    #[test]
    fn score_in_range() {
        let v = IrexVerifier::new(0.5);
        let probe = make_entry("p1", 42);
        let gallery = make_entry("g1", 80);
        let r = v.verify(&probe, &gallery);
        assert!(
            (0.0..=1.0).contains(&r.score),
            "score {} out of range",
            r.score
        );
    }

    #[test]
    fn identical_templates_score_one() {
        let v = IrexVerifier::new(0.5);
        let entry = make_entry("sub_1_a", 100);
        let r = v.verify(&entry, &entry);
        assert!(
            (r.score - 1.0).abs() < 1e-4,
            "expected ~1.0, got {}",
            r.score
        );
    }

    #[test]
    fn decision_true_when_score_above_threshold() {
        let threshold = 0.3;
        let v = IrexVerifier::new(threshold);
        let entry = make_entry("sub_1_a", 100);
        let r = v.verify(&entry, &entry); // identical -> score = 1.0
        assert!(r.decision);
    }

    #[test]
    fn decision_false_when_score_below_threshold() {
        let threshold = 0.99;
        let v = IrexVerifier::new(threshold);
        let probe = make_entry("p1", 0x00);
        let gallery = make_entry("g1", 0xFF);
        let r = v.verify(&probe, &gallery);
        assert!(!r.decision, "expected no-match for flipped bits");
    }

    #[test]
    fn result_ids_match_entries() {
        let v = IrexVerifier::new(0.5);
        let probe = make_entry("probe_1", 10);
        let gallery = make_entry("gallery_1", 20);
        let r = v.verify(&probe, &gallery);
        assert_eq!(r.probe_id, "probe_1");
        assert_eq!(r.gallery_id, "gallery_1");
    }
}
