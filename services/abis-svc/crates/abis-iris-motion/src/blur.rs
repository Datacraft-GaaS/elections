//! Motion blur estimation via Laplacian variance.
//!
//! A high-variance Laplacian response indicates a sharp image (low blur score).
//! A low-variance response indicates blur or uniform regions (high blur score).

use eel_abis_iris_segment::GrayImage;
use serde::{Deserialize, Serialize};

/// Result of blur estimation for a single frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlurResult {
    /// Blur score in [0, 1]. 0.0 = perfectly sharp, 1.0 = maximally blurred.
    pub blur_score: f32,
    /// True when blur_score < max_acceptable_blur threshold.
    pub is_acceptable: bool,
}

/// Estimates motion blur from a grayscale image using Laplacian variance.
#[derive(Debug, Clone)]
pub struct BlurEstimator {
    /// Maximum blur score that is still considered acceptable. Default 0.5.
    pub max_acceptable_blur: f32,
}

impl Default for BlurEstimator {
    fn default() -> Self {
        Self {
            max_acceptable_blur: 0.5,
        }
    }
}

impl BlurEstimator {
    pub fn new(max_acceptable_blur: f32) -> Self {
        Self {
            max_acceptable_blur,
        }
    }

    /// Estimate blur by computing the Laplacian variance across the whole image.
    ///
    /// The discrete Laplacian at pixel (x,y) is approximated as:
    ///   L(x,y) = 4*c - left - right - top - bottom
    /// where out-of-bounds neighbours are clamped to the pixel itself.
    pub fn estimate(&self, image: &GrayImage) -> BlurResult {
        let laplacian_var = laplacian_variance(image);
        // Normalise: maximum achievable Laplacian variance for 8-bit pixels is ~1020.
        // blur_score is inverse of sharpness: high variance -> low blur.
        let blur_score = (1.0 - (laplacian_var / 1020.0).clamp(0.0, 1.0)).clamp(0.0, 1.0);
        let is_acceptable = blur_score < self.max_acceptable_blur;
        BlurResult {
            blur_score,
            is_acceptable,
        }
    }
}

/// Compute the variance of the discrete Laplacian response.
fn laplacian_variance(image: &GrayImage) -> f32 {
    let w = image.width;
    let h = image.height;
    let n = (w * h) as usize;
    if n == 0 {
        return 0.0;
    }

    let lap_values: Vec<f32> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| {
            let center = image.get(x, y) as i32;
            let left = image.get(x.saturating_sub(1), y) as i32;
            let right = image.get(x.min(w - 1).saturating_add(1).min(w - 1), y) as i32;
            let top = image.get(x, y.saturating_sub(1)) as i32;
            let bottom = image.get(x, y.min(h - 1).saturating_add(1).min(h - 1)) as i32;
            (4 * center - left - right - top - bottom) as f32
        })
        .collect();

    let mean = lap_values.iter().sum::<f32>() / n as f32;
    let var = lap_values
        .iter()
        .map(|v| (v - mean) * (v - mean))
        .sum::<f32>()
        / n as f32;
    var
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_image(val: u8) -> GrayImage {
        GrayImage::from_raw(16, 16, vec![val; 256])
    }

    fn checkerboard_image() -> GrayImage {
        let data: Vec<u8> = (0..256)
            .map(|i| {
                let x = i % 16u32;
                let y = i / 16u32;
                if (x + y) % 2 == 0 {
                    255
                } else {
                    0
                }
            })
            .collect();
        GrayImage::from_raw(16, 16, data)
    }

    #[test]
    fn blur_score_in_unit_interval() {
        let estimator = BlurEstimator::default();
        for val in [0u8, 64, 128, 192, 255] {
            let img = flat_image(val);
            let r = estimator.estimate(&img);
            assert!(
                r.blur_score >= 0.0 && r.blur_score <= 1.0,
                "score out of range: {}",
                r.blur_score
            );
        }
    }

    #[test]
    fn flat_image_is_not_acceptable() {
        // A uniform image has zero Laplacian variance, so blur_score = 1.0 -> not acceptable.
        let estimator = BlurEstimator::default();
        let img = flat_image(128);
        let r = estimator.estimate(&img);
        assert!(
            !r.is_acceptable,
            "flat image should not be acceptable (blur_score={})",
            r.blur_score
        );
    }

    #[test]
    fn high_variance_image_is_acceptable() {
        // A checkerboard has maximum Laplacian response -> low blur_score -> acceptable.
        let estimator = BlurEstimator::default();
        let img = checkerboard_image();
        let r = estimator.estimate(&img);
        assert!(
            r.is_acceptable,
            "checkerboard should be acceptable (blur_score={})",
            r.blur_score
        );
    }

    #[test]
    fn blur_score_higher_for_flat_than_checkerboard() {
        let estimator = BlurEstimator::default();
        let flat = estimator.estimate(&flat_image(128));
        let sharp = estimator.estimate(&checkerboard_image());
        assert!(
            flat.blur_score > sharp.blur_score,
            "flat({}) should have higher blur than sharp({})",
            flat.blur_score,
            sharp.blur_score
        );
    }

    #[test]
    fn custom_threshold_changes_acceptability() {
        let strict = BlurEstimator::new(0.0);
        let img = checkerboard_image();
        // Even a sharp image is not acceptable with threshold=0.
        let r = strict.estimate(&img);
        assert!(!r.is_acceptable);
    }
}
