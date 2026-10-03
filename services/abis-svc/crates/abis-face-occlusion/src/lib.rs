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

//! Occlusion-robust face matching.
//!
//! References:
//! - Song, L. et al. (2019). "Occlusion Robust Face Recognition Based on
//!   Mask Learning with Pairwise Differential Siamese Network." ICCV 2019.
//! - Zhao, J. et al. (2018). "Towards Pose Invariant Face Recognition in the
//!   Wild." CVPR 2018. (attention-weighted embedding approach)
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const EMBED_DIM: usize = 512;

/// Occlusion detection error.
#[derive(Debug, Error)]
pub enum OcclusionError {
    #[error("image is empty")]
    EmptyImage,
    #[error("embedding dimension mismatch")]
    DimensionMismatch,
}

/// Occlusion region type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OcclusionType {
    /// Surgical or cloth mask covering lower face.
    Mask,
    /// Eyeglasses (sunglasses or prescription frames).
    Glasses,
    /// Head covering obscuring forehead/sides.
    Hijab,
    /// Hand covering part of face.
    Hand,
    /// Other/unclassified occlusion.
    Other,
    /// No occlusion detected.
    None,
}

/// Per-region occlusion detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionRegion {
    pub region_type: OcclusionType,
    /// Fraction of the face region covered by this occlusion [0.0, 1.0].
    pub coverage: f32,
}

/// Face occlusion detector using skin-tone segmentation heuristics.
///
/// Reference: Klum et al. (2014). "Soft biometric attributes from facial shape."
/// IEEE BTAS 2014.
#[derive(Debug, Clone, Default)]
pub struct OcclusionDetector;

impl OcclusionDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect occlusion regions in an aligned 112x112 RGB image (3 bytes/pixel).
    ///
    /// Uses skin-tone presence in three vertical bands to classify the region
    /// type: top-third (Hijab), middle-third (Glasses), bottom-third (Mask).
    pub fn detect(
        &self,
        rgb: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<OcclusionRegion>, OcclusionError> {
        if rgb.is_empty() || width == 0 || height == 0 {
            return Err(OcclusionError::EmptyImage);
        }
        let w = width as usize;
        let h = height as usize;
        let third = h / 3;

        let skin_fraction = |y_start: usize, y_end: usize| -> f32 {
            let total = w * (y_end - y_start);
            if total == 0 {
                return 0.0;
            }
            let skin: usize = (y_start..y_end)
                .flat_map(|y| (0..w).map(move |x| (y, x)))
                .filter(|&(y, x)| {
                    let idx = (y * w + x) * 3;
                    if idx + 2 >= rgb.len() {
                        return false;
                    }
                    let r = rgb[idx] as i32;
                    let g = rgb[idx + 1] as i32;
                    let b = rgb[idx + 2] as i32;
                    r > 80 && r < 240 && r > g && g > b && (r - b) > 15
                })
                .count();
            skin as f32 / total as f32
        };

        let top_skin = skin_fraction(0, third);
        let mid_skin = skin_fraction(third, 2 * third);
        let bottom_skin = skin_fraction(2 * third, h);

        let mut regions = Vec::new();

        // Bottom band occlusion: low skin => mask.
        if bottom_skin < 0.2 {
            regions.push(OcclusionRegion {
                region_type: OcclusionType::Mask,
                coverage: 1.0 - bottom_skin,
            });
        }
        // Middle band: glasses occlude the eye area.
        if mid_skin < 0.25 {
            regions.push(OcclusionRegion {
                region_type: OcclusionType::Glasses,
                coverage: 1.0 - mid_skin,
            });
        }
        // Top band: hijab/hat.
        if top_skin < 0.15 {
            regions.push(OcclusionRegion {
                region_type: OcclusionType::Hijab,
                coverage: 1.0 - top_skin,
            });
        }
        if regions.is_empty() {
            regions.push(OcclusionRegion {
                region_type: OcclusionType::None,
                coverage: 0.0,
            });
        }
        Ok(regions)
    }

    /// Overall occlusion fraction [0.0, 1.0].
    pub fn occlusion_fraction(&self, regions: &[OcclusionRegion]) -> f32 {
        regions
            .iter()
            .filter(|r| r.region_type != OcclusionType::None)
            .map(|r| r.coverage / 3.0) // normalise by 3 regions
            .sum::<f32>()
            .clamp(0.0, 1.0)
    }
}

/// Occlusion-robust matcher: applies per-dimension attention gates based on
/// the detected occlusion mask to suppress corrupted embedding dimensions.
///
/// Reference: Song et al. (2019). ICCV 2019.
#[derive(Debug, Clone)]
pub struct OcclusionRobustMatcher {
    /// Minimum cosine similarity for a match.
    pub threshold: f32,
}

impl Default for OcclusionRobustMatcher {
    fn default() -> Self {
        Self { threshold: 0.5 }
    }
}

impl OcclusionRobustMatcher {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    /// Build an attention gate from the occlusion regions.
    ///
    /// Dimensions corresponding to occluded regions receive reduced weight.
    /// Bottom-third occlusion (Mask) gates out the last 170 dimensions,
    /// middle-third (Glasses) gates the middle 170, top-third the first 170.
    pub fn build_gate(&self, regions: &[OcclusionRegion]) -> Vec<f32> {
        let mut gate = vec![1.0f32; EMBED_DIM];
        let seg = EMBED_DIM / 3; // ~170 dims per region

        for region in regions {
            let (start, end, weight) = match region.region_type {
                OcclusionType::Hijab => (0, seg, (1.0 - region.coverage).max(0.1)),
                OcclusionType::Glasses => (seg, 2 * seg, (1.0 - region.coverage).max(0.1)),
                OcclusionType::Mask => (2 * seg, EMBED_DIM, (1.0 - region.coverage).max(0.1)),
                OcclusionType::Hand => (0, EMBED_DIM, (1.0 - region.coverage * 0.5).max(0.1)),
                OcclusionType::Other => (0, EMBED_DIM, 0.8),
                OcclusionType::None => continue,
            };
            for g in &mut gate[start..end] {
                *g = g.min(weight);
            }
        }
        gate
    }

    /// Masked cosine similarity: multiply both embeddings element-wise by the
    /// gate before computing cosine similarity.
    pub fn masked_similarity(
        &self,
        probe: &[f32],
        gallery: &[f32],
        gate: &[f32],
    ) -> Result<f32, OcclusionError> {
        if probe.len() != EMBED_DIM || gallery.len() != EMBED_DIM || gate.len() != EMBED_DIM {
            return Err(OcclusionError::DimensionMismatch);
        }
        let gated_probe: Vec<f32> = probe.iter().zip(gate.iter()).map(|(p, g)| p * g).collect();
        let gated_gallery: Vec<f32> = gallery
            .iter()
            .zip(gate.iter())
            .map(|(g, w)| g * w)
            .collect();

        let norm_p: f32 = gated_probe
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt()
            .max(1e-9);
        let norm_g: f32 = gated_gallery
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt()
            .max(1e-9);

        let dot: f32 = gated_probe
            .iter()
            .zip(gated_gallery.iter())
            .map(|(a, b)| a * b)
            .sum();
        Ok((dot / (norm_p * norm_g)).clamp(-1.0, 1.0))
    }
}
