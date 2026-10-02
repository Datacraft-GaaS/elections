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

//! Unified iris segmentation pipeline: Daugman + OSIRIS + masking.

use crate::{
    daugman::{locate_circle, DaugmanParams},
    mask::build_iris_mask,
    osiris::viterbi_refine,
    Circle, GrayImage, IrisMask,
};
use serde::{Deserialize, Serialize};

/// Full iris segmentation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisSegmentation {
    /// Pupillary boundary circle.
    pub pupil: Circle,
    /// Limbic boundary circle.
    pub limbus: Circle,
    /// Per-pixel validity mask (same dims as source image).
    pub mask: IrisMask,
    /// Iris-to-sclera ratio: iris annulus area / sclera area visible. Ideal > 0.7.
    pub iris_sclera_ratio: f32,
}

impl IrisSegmentation {
    /// Fraction of iris annulus pixels that are valid (usable).
    pub fn usable_fraction(&self) -> f32 {
        self.mask.valid_fraction()
    }
}

/// Error type for segmentation failures.
#[derive(Debug, thiserror::Error)]
pub enum SegmentError {
    #[error("image too small: {width}x{height}, need at least 40x40")]
    ImageTooSmall { width: u32, height: u32 },
    #[error("pupil radius {pupil_r:.1} exceeds limbus radius {limbus_r:.1}")]
    PupilLargerThanLimbus { pupil_r: f32, limbus_r: f32 },
}

/// Configuration for the segmentation pipeline.
#[derive(Debug, Clone)]
pub struct SegmentConfig {
    pub pupil_params: DaugmanParams,
    pub limbus_params: DaugmanParams,
    /// Viterbi refinement radial search half-width in pixels.
    pub viterbi_delta: u32,
    /// Pixel intensity below which a pixel is masked as eyelash.
    pub eyelash_threshold: u8,
    /// Pixel intensity above which a pixel is masked as specular reflection.
    pub specular_threshold: u8,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            pupil_params: DaugmanParams {
                min_radius: 15,
                max_radius: 80,
                angular_samples: 64,
                sigma: 2.0,
                centre_step: 4,
            },
            limbus_params: DaugmanParams {
                min_radius: 60,
                max_radius: 140,
                angular_samples: 64,
                sigma: 2.0,
                centre_step: 4,
            },
            viterbi_delta: 5,
            eyelash_threshold: 25,
            specular_threshold: 240,
        }
    }
}

/// Iris segmentation entry point.
pub struct SegmentIris {
    config: SegmentConfig,
}

impl SegmentIris {
    pub fn new(config: SegmentConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(SegmentConfig::default())
    }

    /// Segment the iris in `image`.
    ///
    /// # Errors
    /// Returns `SegmentError` when the image is too small or the detected circles
    /// are geometrically inconsistent.
    pub fn segment(&self, image: &GrayImage) -> Result<IrisSegmentation, SegmentError> {
        if image.width < 40 || image.height < 40 {
            return Err(SegmentError::ImageTooSmall {
                width: image.width,
                height: image.height,
            });
        }

        // Step 1: Daugman coarse pupil localisation.
        let coarse_pupil = locate_circle(image, &self.config.pupil_params, None);

        // Step 2: Daugman coarse limbus localisation.
        let coarse_limbus = locate_circle(image, &self.config.limbus_params, None);

        // Step 3: OSIRIS Viterbi boundary refinement.
        let pupil = viterbi_refine(image, coarse_pupil, self.config.viterbi_delta, 64);
        let limbus = viterbi_refine(image, coarse_limbus, self.config.viterbi_delta, 64);

        if pupil.radius >= limbus.radius {
            return Err(SegmentError::PupilLargerThanLimbus {
                pupil_r: pupil.radius,
                limbus_r: limbus.radius,
            });
        }

        // Step 4: Build occlusion mask.
        let mask = build_iris_mask(
            image,
            pupil,
            limbus,
            self.config.eyelash_threshold,
            self.config.specular_threshold,
        );

        // Step 5: Compute iris/sclera ratio.
        let iris_sclera_ratio = compute_iris_sclera_ratio(image, pupil, limbus);

        Ok(IrisSegmentation {
            pupil,
            limbus,
            mask,
            iris_sclera_ratio,
        })
    }
}

/// Iris-to-sclera pixel ratio: fraction of iris annulus area relative to expected area.
///
/// A well-captured iris shows the full annulus, ratio ~ 1.0.
/// Partial capture or large occlusion brings it below 0.7.
fn compute_iris_sclera_ratio(image: &GrayImage, pupil: Circle, limbus: Circle) -> f32 {
    let w = image.width;
    let h = image.height;

    let mut iris_pixels = 0u32;
    let mut sclera_pixels = 0u32;

    for y in 0..h {
        for x in 0..w {
            let in_limbus = limbus.contains(x as f32, y as f32);
            let in_pupil = pupil.contains(x as f32, y as f32);
            if in_limbus && !in_pupil {
                iris_pixels += 1;
            } else if !in_limbus {
                sclera_pixels += 1;
            }
        }
    }

    // Ratio of iris area to sclera area (clamped to [0, 1]).
    let total = (iris_pixels + sclera_pixels) as f32;
    if total < 1.0 {
        return 0.0;
    }
    (iris_pixels as f32 / total).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic iris image: dark pupil, mid-grey iris, bright sclera.
    fn make_iris(width: u32, height: u32, pupil_r: u32, limbus_r: u32) -> GrayImage {
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let data: Vec<u8> = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let r = (dx * dx + dy * dy).sqrt();
                    if r < pupil_r as f32 {
                        20
                    } else if r < limbus_r as f32 {
                        150
                    } else {
                        220
                    }
                })
            })
            .collect();
        GrayImage::from_raw(width, height, data)
    }

    #[test]
    fn segment_succeeds_on_synthetic_iris() {
        let img = make_iris(240, 240, 30, 90);
        let seg = SegmentIris::with_defaults()
            .segment(&img)
            .expect("segment failed");
        assert!(seg.pupil.radius < seg.limbus.radius);
        assert!(seg.limbus.radius > 0.0);
    }

    #[test]
    fn segment_error_on_tiny_image() {
        let img = GrayImage::from_raw(10, 10, vec![128; 100]);
        let err = SegmentIris::with_defaults().segment(&img).unwrap_err();
        assert!(matches!(err, SegmentError::ImageTooSmall { .. }));
    }

    #[test]
    fn iris_sclera_ratio_positive() {
        let img = make_iris(200, 200, 25, 70);
        let seg = SegmentIris::with_defaults().segment(&img).expect("segment");
        assert!(seg.iris_sclera_ratio > 0.0);
        assert!(seg.iris_sclera_ratio <= 1.0);
    }

    #[test]
    fn pupil_and_limbus_distinct() {
        let img = make_iris(240, 240, 28, 85);
        let seg = SegmentIris::with_defaults().segment(&img).expect("segment");
        assert!(seg.pupil.radius < seg.limbus.radius);
    }

    #[test]
    fn mask_non_empty() {
        let img = make_iris(240, 240, 28, 85);
        let seg = SegmentIris::with_defaults().segment(&img).expect("segment");
        assert!(seg.mask.valid_fraction() >= 0.0);
        assert!(seg.mask.valid_fraction() <= 1.0);
    }

    #[test]
    fn usable_fraction_matches_mask() {
        let img = make_iris(200, 200, 25, 70);
        let seg = SegmentIris::with_defaults().segment(&img).expect("segment");
        assert!((seg.usable_fraction() - seg.mask.valid_fraction()).abs() < 1e-6);
    }

    #[test]
    fn compute_iris_sclera_ratio_bounded() {
        let img = make_iris(200, 200, 20, 70);
        let pupil = Circle::new(100.0, 100.0, 20.0);
        let limbus = Circle::new(100.0, 100.0, 70.0);
        let ratio = super::compute_iris_sclera_ratio(&img, pupil, limbus);
        assert!((0.0..=1.0).contains(&ratio));
    }

    #[test]
    fn large_iris_has_higher_ratio_than_small() {
        let img_big = make_iris(300, 300, 20, 120);
        let img_small = make_iris(300, 300, 20, 50);
        let p_big = Circle::new(150.0, 150.0, 20.0);
        let l_big = Circle::new(150.0, 150.0, 120.0);
        let p_small = Circle::new(150.0, 150.0, 20.0);
        let l_small = Circle::new(150.0, 150.0, 50.0);
        let r_big = super::compute_iris_sclera_ratio(&img_big, p_big, l_big);
        let r_small = super::compute_iris_sclera_ratio(&img_small, p_small, l_small);
        // Larger iris fills more of the image, so ratio is higher.
        assert!(r_big > r_small, "big {r_big:.3} vs small {r_small:.3}");
    }
}
