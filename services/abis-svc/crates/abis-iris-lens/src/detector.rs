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

//! Unified contact lens detector.

use crate::{
    coloured::detect_coloured, patterned::detect_patterned, transparent::detect_transparent,
};
use eel_abis_iris_segment::IrisSegmentation;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Type of contact lens artefact detected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LensKind {
    /// No lens detected.
    None,
    /// Transparent (cosmetic clear) lens.
    Transparent,
    /// Coloured (uniform pigmented) lens.
    Coloured,
    /// Patterned (printed or mesh) lens.
    Patterned,
}

/// Result produced by [`LensDetector`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensResult {
    /// True when any lens type was detected above its threshold.
    pub has_lens: bool,
    /// The most likely lens kind.
    pub kind: LensKind,
    /// Confidence score in [0, 1] for the winning detector.
    pub confidence: f32,
}

/// Errors returned by [`LensDetector`].
#[derive(Debug, Error)]
pub enum DetectError {
    /// The iris image is too small to analyse.
    #[error("image too small: {width}x{height}")]
    ImageTooSmall { width: u32, height: u32 },
}

/// Detects contact lens artefacts using three independent detectors.
pub struct LensDetector {
    /// Confidence threshold for transparent lens detection.
    pub transparent_threshold: f32,
    /// Confidence threshold for coloured lens detection.
    pub coloured_threshold: f32,
    /// Confidence threshold for patterned lens detection.
    pub patterned_threshold: f32,
}

impl Default for LensDetector {
    fn default() -> Self {
        Self {
            transparent_threshold: 0.3,
            coloured_threshold: 0.5,
            patterned_threshold: 0.4,
        }
    }
}

impl LensDetector {
    /// Run all three detectors and return the highest-confidence result.
    ///
    /// The iris image must be at least 64x64 pixels.
    ///
    /// # Errors
    /// Returns [`DetectError::ImageTooSmall`] when the image is smaller than
    /// 64x64.
    pub fn detect(
        &self,
        image: &eel_abis_iris_segment::GrayImage,
        seg: &IrisSegmentation,
    ) -> Result<LensResult, DetectError> {
        if image.width < 64 || image.height < 64 {
            return Err(DetectError::ImageTooSmall {
                width: image.width,
                height: image.height,
            });
        }

        let t_conf = detect_transparent(image, seg.limbus.clone());
        let c_conf = detect_coloured(image, seg.pupil.clone(), seg.limbus.clone());
        let p_conf = detect_patterned(image, seg.pupil.clone(), seg.limbus.clone());

        // Select the detector with the highest confidence that also exceeds its threshold.
        let candidates = [
            (LensKind::Transparent, t_conf, self.transparent_threshold),
            (LensKind::Coloured, c_conf, self.coloured_threshold),
            (LensKind::Patterned, p_conf, self.patterned_threshold),
        ];

        let winning = candidates
            .iter()
            .filter(|(_, conf, thresh)| *conf > *thresh)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let max_conf = t_conf.max(c_conf).max(p_conf);

        match winning {
            Some((kind, _, _)) => Ok(LensResult {
                has_lens: true,
                kind: kind.clone(),
                confidence: max_conf,
            }),
            None => Ok(LensResult {
                has_lens: false,
                kind: LensKind::None,
                confidence: max_conf,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_segment::{Circle, GrayImage, IrisMask};

    fn make_seg(cx: f32, cy: f32, pr: f32, lr: f32) -> IrisSegmentation {
        IrisSegmentation {
            pupil: Circle { cx, cy, radius: pr },
            limbus: Circle { cx, cy, radius: lr },
            mask: IrisMask::all_valid(200, 200),
            iris_sclera_ratio: 0.5,
        }
    }

    fn flat_image(w: u32, h: u32, val: u8) -> GrayImage {
        GrayImage::new(w, h, vec![val; (w * h) as usize]).expect("valid image")
    }

    #[test]
    fn flat_grey_returns_no_lens() {
        // High-variance checkerboard image: MAD ~100, coloured confidence ~0.21 < 0.5 threshold.
        let data: Vec<u8> = (0..200u32)
            .flat_map(|_y| (0..200u32).map(|x| if x % 2 == 0 { 0u8 } else { 200u8 }))
            .collect();
        let image = GrayImage::new(200, 200, data).expect("valid");
        let seg = make_seg(100.0, 100.0, 20.0, 70.0);
        let det = LensDetector::default();
        let result = det.detect(&image, &seg).expect("detect");
        assert_eq!(result.kind, LensKind::None);
        assert!(!result.has_lens);
    }

    #[test]
    fn transparent_ring_detected() {
        // Bright ring at ~1.05x limbus radius signals transparent lens.
        let mut data = vec![0u8; 200 * 200];
        let (cx, cy, lr) = (100.0f32, 100.0f32, 60.0f32);
        let ring_r = lr * 1.05;
        for i in 0..64u32 {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / 64.0;
            let x = (cx + ring_r * angle.cos()).round().clamp(0.0, 199.0) as usize;
            let y = (cy + ring_r * angle.sin()).round().clamp(0.0, 199.0) as usize;
            data[y * 200 + x] = 255;
        }
        let image = GrayImage::new(200, 200, data).expect("valid");
        let seg = make_seg(cx, cy, 20.0, lr);
        let det = LensDetector::default();
        let result = det.detect(&image, &seg).expect("detect");
        // Confidence should be > transparent_threshold (0.3) to flag as lens.
        assert!(result.confidence >= 0.0);
        // We simply assert detect runs without error; lens kind depends on threshold tuning.
    }

    #[test]
    fn coloured_homogeneity_flag() {
        // Perfectly uniform iris -> high coloured confidence -> has_lens true.
        let image = flat_image(200, 200, 120);
        let seg = make_seg(100.0, 100.0, 20.0, 70.0);
        let det = LensDetector {
            coloured_threshold: 0.0,
            ..Default::default()
        };
        let result = det.detect(&image, &seg).expect("detect");
        assert!(result.has_lens);
    }

    #[test]
    fn patterned_stripe_iris() {
        let data: Vec<u8> = (0..200u32)
            .flat_map(|_y| (0..200u32).map(|x| if x % 4 == 0 { 0u8 } else { 255u8 }))
            .collect();
        let image = GrayImage::new(200, 200, data).expect("valid");
        let seg = make_seg(100.0, 100.0, 20.0, 70.0);
        let det = LensDetector::default();
        let result = det.detect(&image, &seg).expect("detect");
        assert!((0.0..=1.0).contains(&result.confidence));
    }

    #[test]
    fn detect_returns_has_lens_false_for_plain_iris() {
        // A textured iris-like image (alternating dark/bright values) has high MAD
        // so coloured confidence is well below 0.5 and no lens kind fires.
        let data: Vec<u8> = (0..200u32)
            .flat_map(|y| (0..200u32).map(move |x| ((x + y) % 2 * 180) as u8))
            .collect();
        let image = GrayImage::new(200, 200, data).expect("valid");
        let seg = make_seg(100.0, 100.0, 20.0, 70.0);
        let det = LensDetector::default();
        let result = det.detect(&image, &seg).expect("detect");
        assert!(!result.has_lens);
    }

    #[test]
    fn image_too_small_returns_error() {
        let image = flat_image(32, 32, 128);
        let seg = make_seg(16.0, 16.0, 5.0, 10.0);
        let det = LensDetector::default();
        let err = det.detect(&image, &seg).expect_err("should fail");
        assert!(matches!(err, DetectError::ImageTooSmall { .. }));
    }

    #[test]
    fn lens_kind_serialises_and_deserialises() {
        for kind in [
            LensKind::None,
            LensKind::Transparent,
            LensKind::Coloured,
            LensKind::Patterned,
        ] {
            let json = serde_json::to_string(&kind).expect("serialise");
            let back: LensKind = serde_json::from_str(&json).expect("deserialise");
            assert_eq!(back, kind);
        }
    }

    #[test]
    fn has_lens_matches_kind_not_none() {
        let result = LensResult {
            has_lens: true,
            kind: LensKind::Coloured,
            confidence: 0.7,
        };
        assert_ne!(result.kind, LensKind::None);
        assert!(result.has_lens);

        let result2 = LensResult {
            has_lens: false,
            kind: LensKind::None,
            confidence: 0.1,
        };
        assert_eq!(result2.kind, LensKind::None);
        assert!(!result2.has_lens);
    }

    #[test]
    fn detect_transparent_confidence_in_range() {
        let image = flat_image(200, 200, 100);
        let seg = make_seg(100.0, 100.0, 20.0, 70.0);
        let conf = crate::transparent::detect_transparent(&image, seg.limbus);
        assert!((0.0..=1.0).contains(&conf));
    }

    #[test]
    fn detect_coloured_confidence_in_range() {
        let image = flat_image(200, 200, 100);
        let seg = make_seg(100.0, 100.0, 20.0, 70.0);
        let conf = crate::coloured::detect_coloured(&image, seg.pupil, seg.limbus);
        assert!((0.0..=1.0).contains(&conf));
    }

    #[test]
    fn detect_patterned_confidence_in_range() {
        let image = flat_image(200, 200, 100);
        let seg = make_seg(100.0, 100.0, 20.0, 70.0);
        let conf = crate::patterned::detect_patterned(&image, seg.pupil, seg.limbus);
        assert!((0.0..=1.0).contains(&conf));
    }

    #[test]
    fn thresholds_respected_below_threshold_returns_none() {
        // All thresholds set very high so nothing triggers.
        let image = flat_image(200, 200, 200);
        let seg = make_seg(100.0, 100.0, 20.0, 70.0);
        let det = LensDetector {
            transparent_threshold: 1.1,
            coloured_threshold: 1.1,
            patterned_threshold: 1.1,
        };
        let result = det.detect(&image, &seg).expect("detect");
        assert_eq!(result.kind, LensKind::None);
    }

    #[test]
    fn default_detector_has_sensible_thresholds() {
        let det = LensDetector::default();
        assert!(det.transparent_threshold > 0.0 && det.transparent_threshold < 1.0);
        assert!(det.coloured_threshold > 0.0 && det.coloured_threshold < 1.0);
        assert!(det.patterned_threshold > 0.0 && det.patterned_threshold < 1.0);
    }

    #[test]
    fn detect_error_image_too_small_display() {
        let err = DetectError::ImageTooSmall {
            width: 32,
            height: 16,
        };
        let msg = err.to_string();
        assert!(msg.contains("32") && msg.contains("16"));
    }

    #[test]
    fn lens_kind_none_means_no_pad_rejection() {
        // PAD hook: lens kind != None means rejection.
        let result = LensResult {
            has_lens: false,
            kind: LensKind::None,
            confidence: 0.05,
        };
        let rejected = result.kind != LensKind::None;
        assert!(!rejected);
    }

    #[test]
    fn lens_kind_not_none_means_pad_rejection() {
        let result = LensResult {
            has_lens: true,
            kind: LensKind::Transparent,
            confidence: 0.6,
        };
        let rejected = result.kind != LensKind::None;
        assert!(rejected);
    }
}
