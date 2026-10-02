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

//! TextureMatcher: fuses FingerCode, LBP, and MB-LBP scores with configurable weights.
//!
//! Intended as a fallback when minutiae extraction quality is low (NFIQ2 <= 2).

use crate::{
    fingercode::{
        encode_fingercode, filter_bank, locate_reference_point, match_fingercode, tessellate,
    },
    lbp::{chi_square_similarity, lbp_histogram_for_image},
    mb_lbp::{mb_lbp_multi_scale, mb_lbp_similarity},
    GrayImage, TextureError,
};

/// Weights for each sub-matcher in the texture fusion.
#[derive(Debug, Clone)]
pub struct TextureWeights {
    /// Weight for FingerCode (dominant, most principled).
    pub fingercode: f32,
    /// Weight for LBP histogram.
    pub lbp: f32,
    /// Weight for MB-LBP multi-scale histogram.
    pub mb_lbp: f32,
}

impl Default for TextureWeights {
    fn default() -> Self {
        Self {
            fingercode: 0.6,
            lbp: 0.2,
            mb_lbp: 0.2,
        }
    }
}

/// Per-sub-matcher scores returned by TextureMatcher.
#[derive(Debug, Clone)]
pub struct TextureScoreBreakdown {
    pub fingercode: f32,
    pub lbp: f32,
    pub mb_lbp: f32,
    pub fused: f32,
}

/// Texture-based fingerprint matcher fusing FingerCode + LBP + MB-LBP.
#[derive(Debug, Clone)]
pub struct TextureMatcher {
    pub weights: TextureWeights,
    /// Minimum similarity threshold for a 'match' decision.
    pub threshold: f32,
}

impl Default for TextureMatcher {
    fn default() -> Self {
        Self {
            weights: TextureWeights::default(),
            threshold: 0.5,
        }
    }
}

impl TextureMatcher {
    pub fn new(weights: TextureWeights, threshold: f32) -> Self {
        Self { weights, threshold }
    }

    /// Compare two grayscale fingerprint images.
    ///
    /// Returns fused similarity in [0.0, 1.0] and per-component breakdown.
    pub fn compare(
        &self,
        probe: &GrayImage,
        gallery: &GrayImage,
    ) -> Result<TextureScoreBreakdown, TextureError> {
        // FingerCode.
        let fc_score = {
            let rp = locate_reference_point(probe)?;
            let rg = locate_reference_point(gallery)?;
            let sectors_p = tessellate(probe, rp);
            let sectors_g = tessellate(gallery, rg);
            let filtered_p = filter_bank(probe, &[]);
            let filtered_g = filter_bank(gallery, &[]);
            let fcp = encode_fingercode(&sectors_p, &filtered_p);
            let fcg = encode_fingercode(&sectors_g, &filtered_g);
            match_fingercode(&fcp, &fcg)
        };

        // LBP.
        let lbp_score = {
            let hp = lbp_histogram_for_image(probe);
            let hg = lbp_histogram_for_image(gallery);
            chi_square_similarity(&hp, &hg)
        };

        // MB-LBP.
        let mb_score = {
            let hp = mb_lbp_multi_scale(probe);
            let hg = mb_lbp_multi_scale(gallery);
            mb_lbp_similarity(&hp, &hg)
        };

        let w = &self.weights;
        let total_w = w.fingercode + w.lbp + w.mb_lbp;
        let fused = (w.fingercode * fc_score + w.lbp * lbp_score + w.mb_lbp * mb_score) / total_w;

        Ok(TextureScoreBreakdown {
            fingercode: fc_score,
            lbp: lbp_score,
            mb_lbp: mb_score,
            fused,
        })
    }

    /// Return true when the fused score meets the decision threshold.
    pub fn is_match(&self, score: &TextureScoreBreakdown) -> bool {
        score.fused >= self.threshold
    }
}
