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

//! Quality report aggregating all ISO/IEC 29794-6 metrics.

use crate::metrics::{contrast, dilation, gaze_angle, occlusion_pct, sharpness};
use eel_abis_iris_segment::{GrayImage, IrisSegmentation};
use serde::{Deserialize, Serialize};

/// Error from quality assessment.
#[derive(Debug, thiserror::Error)]
pub enum QualityError {
    #[error("image is too small to assess quality: {width}x{height}")]
    ImageTooSmall { width: u32, height: u32 },
}

/// Full quality report per ISO/IEC 29794-6.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    /// Laplacian variance (sharpness), normalised to [0, 1].
    pub sharpness: f32,
    /// Histogram spread (contrast), normalised to [0, 1].
    pub contrast: f32,
    /// Pupil/limbus radius ratio (dilation).
    pub dilation: f32,
    /// Iris annulus area / total visible area (iris_sclera_ratio from segmentation).
    pub iris_sclera_ratio: f32,
    /// Estimated gaze angle in degrees from frontal.
    pub gaze_angle_deg: f32,
    /// Fraction of iris annulus that is occluded (0 = none, 1 = fully occluded).
    pub occlusion_fraction: f32,
    /// Overall quality score in [0, 1] (weighted combination of above).
    pub overall: f32,
}

impl QualityReport {
    /// Compute quality report from image and segmentation.
    ///
    /// # Errors
    /// Returns `QualityError` when the image is too small.
    pub fn compute(image: &GrayImage, seg: &IrisSegmentation) -> Result<Self, QualityError> {
        if image.width < 20 || image.height < 20 {
            return Err(QualityError::ImageTooSmall {
                width: image.width,
                height: image.height,
            });
        }

        let sharpness_v = sharpness(image, seg);
        let contrast_v = contrast(image, seg);
        let dilation_v = dilation(seg);
        let iris_sclera = seg.iris_sclera_ratio;
        let gaze_v = gaze_angle(seg);
        let occlusion_v = occlusion_pct(seg);

        // Weighted combination per ISO 29794-6 scoring model.
        // Weights: sharpness 0.3, contrast 0.2, dilation quality 0.15,
        //          iris_sclera 0.15, gaze quality 0.1, occlusion quality 0.1.
        let dilation_quality = dilation_quality_score(dilation_v);
        let gaze_quality = gaze_quality_score(gaze_v);
        let occlusion_quality = 1.0 - occlusion_v;

        let overall = 0.30 * sharpness_v
            + 0.20 * contrast_v
            + 0.15 * dilation_quality
            + 0.15 * iris_sclera
            + 0.10 * gaze_quality
            + 0.10 * occlusion_quality;

        Ok(Self {
            sharpness: sharpness_v,
            contrast: contrast_v,
            dilation: dilation_v,
            iris_sclera_ratio: iris_sclera,
            gaze_angle_deg: gaze_v,
            occlusion_fraction: occlusion_v,
            overall: overall.clamp(0.0, 1.0),
        })
    }
}

/// Convert dilation ratio to quality score: ideal range 0.3-0.5 = 1.0, else degraded.
fn dilation_quality_score(d: f32) -> f32 {
    if (0.3..=0.5).contains(&d) {
        1.0
    } else if d < 0.3 {
        (d / 0.3).clamp(0.0, 1.0)
    } else {
        // d > 0.5: dilated pupil reduces iris annulus area
        ((1.0 - d) / 0.5).clamp(0.0, 1.0)
    }
}

/// Convert gaze angle to quality score: 0 deg = 1.0, 15 deg = 0.0.
fn gaze_quality_score(angle_deg: f32) -> f32 {
    (1.0 - angle_deg / 15.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_segment::{Circle, IrisMask, IrisSegmentation};

    fn make_iris_image(w: u32, h: u32, pr: f32, lr: f32) -> (GrayImage, IrisSegmentation) {
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let pupil = Circle::new(cx, cy, pr);
        let limbus = Circle::new(cx, cy, lr);
        let mask = IrisMask::all_valid(w, h);
        let data: Vec<u8> = (0..h)
            .flat_map(|y| {
                (0..w).map(move |x| {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let r = (dx * dx + dy * dy).sqrt();
                    if r < pr {
                        30
                    } else if r < lr {
                        150
                    } else {
                        220
                    }
                })
            })
            .collect();
        let image = GrayImage::from_raw(w, h, data);
        let seg = IrisSegmentation {
            pupil,
            limbus,
            mask,
            iris_sclera_ratio: 0.85,
        };
        (image, seg)
    }

    #[test]
    fn quality_report_overall_in_range() {
        let (img, seg) = make_iris_image(200, 200, 25.0, 70.0);
        let report = QualityReport::compute(&img, &seg).expect("quality");
        assert!(
            (0.0..=1.0).contains(&report.overall),
            "overall {}",
            report.overall
        );
    }

    #[test]
    fn all_metrics_in_range() {
        let (img, seg) = make_iris_image(200, 200, 25.0, 70.0);
        let r = QualityReport::compute(&img, &seg).expect("quality");
        assert!((0.0..=1.0).contains(&r.sharpness));
        assert!((0.0..=1.0).contains(&r.contrast));
        assert!((0.0..=1.0).contains(&r.dilation));
        assert!((0.0..=1.0).contains(&r.iris_sclera_ratio));
        assert!(r.gaze_angle_deg >= 0.0);
        assert!((0.0..=1.0).contains(&r.occlusion_fraction));
    }

    #[test]
    fn tiny_image_returns_error() {
        let img = GrayImage::from_raw(10, 10, vec![128; 100]);
        let seg = IrisSegmentation {
            pupil: Circle::new(5.0, 5.0, 2.0),
            limbus: Circle::new(5.0, 5.0, 4.0),
            mask: IrisMask::all_valid(10, 10),
            iris_sclera_ratio: 0.5,
        };
        let err = QualityReport::compute(&img, &seg).unwrap_err();
        assert!(matches!(err, QualityError::ImageTooSmall { .. }));
    }

    #[test]
    fn dilation_quality_ideal_range() {
        assert!((dilation_quality_score(0.4) - 1.0).abs() < 0.01);
        assert!(dilation_quality_score(0.1) < 0.5);
        assert!(dilation_quality_score(0.9) < 0.5);
    }

    #[test]
    fn gaze_quality_frontal_is_one() {
        assert!((gaze_quality_score(0.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn gaze_quality_15deg_is_zero() {
        assert!(gaze_quality_score(15.0).abs() < 0.01);
    }
}
