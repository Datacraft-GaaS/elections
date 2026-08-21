//! Local Binary Pattern (LBP) encoding.
//!
//! Reference: T. Ojala, M. Pietikainen, T. Maenpaa,
//! "Multiresolution gray-scale and rotation invariant texture classification with Local Binary
//! Patterns," IEEE TPAMI, vol. 24, no. 7, pp. 971-987, 2002.
//!
//! Each pixel is encoded by comparing it with its 8 circular neighbors.
//! The result is an 8-bit code used to build a 256-bin histogram over the image.
//! Similarity is measured by the chi-square distance between histograms.

use crate::GrayImage;

/// Number of LBP codes (8-bit LBP -> 256 bins).
pub const LBP_BINS: usize = 256;

/// Compute the LBP code for one pixel at (x, y) using radius-1 neighbours.
pub fn lbp_at(image: &GrayImage, x: i32, y: i32) -> u8 {
    let center = image.pixel(x, y);
    // 8 neighbours in clockwise order starting from (x+1, y).
    let neighbours = [
        image.pixel(x + 1, y),
        image.pixel(x + 1, y + 1),
        image.pixel(x, y + 1),
        image.pixel(x - 1, y + 1),
        image.pixel(x - 1, y),
        image.pixel(x - 1, y - 1),
        image.pixel(x, y - 1),
        image.pixel(x + 1, y - 1),
    ];
    let mut code = 0u8;
    for (i, &n) in neighbours.iter().enumerate() {
        if n >= center {
            code |= 1 << i;
        }
    }
    code
}

/// Build a 256-bin LBP histogram from all interior pixels of an image.
///
/// Border pixels (radius 1) are skipped to avoid out-of-bounds.
pub fn lbp_histogram_for_image(image: &GrayImage) -> Vec<f32> {
    let mut hist = vec![0.0f32; LBP_BINS];
    if image.width < 3 || image.height < 3 {
        return hist;
    }
    for y in 1..(image.height as i32 - 1) {
        for x in 1..(image.width as i32 - 1) {
            let code = lbp_at(image, x, y) as usize;
            hist[code] += 1.0;
        }
    }
    // Normalize to sum 1.
    let total: f32 = hist.iter().sum();
    if total > 0.0 {
        hist.iter_mut().for_each(|v| *v /= total);
    }
    hist
}

/// Build a normalized LBP histogram from raw pixel data.
///
/// Useful when the caller already has a computed LBP map.
pub fn lbp_histogram(lbp_codes: &[u8]) -> Vec<f32> {
    let mut hist = vec![0.0f32; LBP_BINS];
    for &code in lbp_codes {
        hist[code as usize] += 1.0;
    }
    let total: f32 = hist.iter().sum();
    if total > 0.0 {
        hist.iter_mut().for_each(|v| *v /= total);
    }
    hist
}

/// Chi-square distance between two normalized histograms.
///
/// Returns a distance in [0, inf). For matching, lower = more similar.
/// Similarity = 1 / (1 + distance).
pub fn chi_square_distance(h1: &[f32], h2: &[f32]) -> f32 {
    assert_eq!(h1.len(), h2.len(), "histogram length mismatch");
    h1.iter()
        .zip(h2.iter())
        .map(|(&a, &b)| {
            let s = a + b;
            if s < f32::EPSILON {
                0.0
            } else {
                (a - b).powi(2) / s
            }
        })
        .sum::<f32>()
        * 0.5
}

/// Convert chi-square distance to a similarity score in [0.0, 1.0].
pub fn chi_square_similarity(h1: &[f32], h2: &[f32]) -> f32 {
    let d = chi_square_distance(h1, h2);
    1.0 / (1.0 + d)
}
