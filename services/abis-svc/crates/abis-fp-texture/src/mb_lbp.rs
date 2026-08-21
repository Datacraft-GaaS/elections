//! Multi-Block Local Binary Pattern (MB-LBP) encoding.
//!
//! Reference: L. Zhang, R. Chu, S. Xiang, S. Liao, S. Z. Li,
//! "Face Detection Based on Multi-Block LBP Representation," IAPR/IEEE Int. Conf.
//! Biometrics, 2007. Applied to fingerprint texture for scale invariance.
//!
//! Instead of single pixels, block averages are compared. Three block sizes
//! (3x3, 5x5, 9x9) capture texture at multiple scales.

use crate::GrayImage;

/// Supported block sizes for MB-LBP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MbLbpBlockSize {
    S3 = 3,
    S5 = 5,
    S9 = 9,
}

impl MbLbpBlockSize {
    pub fn pixels(self) -> u32 {
        self as u32
    }
}

/// Compute the average intensity of a rectangular block.
fn block_mean(image: &GrayImage, x: i32, y: i32, b: u32) -> f32 {
    let mut sum = 0u32;
    let mut count = 0u32;
    for dy in 0..b as i32 {
        for dx in 0..b as i32 {
            let px = x + dx;
            let py = y + dy;
            if px >= 0 && py >= 0 && px < image.width as i32 && py < image.height as i32 {
                sum += image.pixel(px, py) as u32;
                count += 1;
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        sum as f32 / count as f32
    }
}

/// Compute the MB-LBP code at position (x, y) using block size b.
///
/// The center block occupies [x, x+b) x [y, y+b).
/// The 8 neighbouring blocks are arranged around it.
pub fn mb_lbp_at(image: &GrayImage, x: i32, y: i32, b: u32) -> u8 {
    let b = b as i32;
    let center = block_mean(image, x, y, b as u32);
    let neighbours = [
        block_mean(image, x + b, y, b as u32),
        block_mean(image, x + b, y + b, b as u32),
        block_mean(image, x, y + b, b as u32),
        block_mean(image, x - b, y + b, b as u32),
        block_mean(image, x - b, y, b as u32),
        block_mean(image, x - b, y - b, b as u32),
        block_mean(image, x, y - b, b as u32),
        block_mean(image, x + b, y - b, b as u32),
    ];
    let mut code = 0u8;
    for (i, &n) in neighbours.iter().enumerate() {
        if n >= center {
            code |= 1 << i;
        }
    }
    code
}

/// Build a normalized 256-bin MB-LBP histogram for the given block size.
///
/// The step between sampled positions equals the block size (non-overlapping grid).
pub fn mb_lbp_histogram(image: &GrayImage, block_size: MbLbpBlockSize) -> Vec<f32> {
    let mut hist = vec![0.0f32; 256];
    let b = block_size.pixels() as i32;
    let w = image.width as i32;
    let h = image.height as i32;

    // Sample at block-step intervals, keeping a margin of one block on each side.
    let mut count = 0u32;
    let mut x = b;
    while x + 2 * b <= w {
        let mut y = b;
        while y + 2 * b <= h {
            let code = mb_lbp_at(image, x, y, block_size.pixels()) as usize;
            hist[code] += 1.0;
            count += 1;
            y += b;
        }
        x += b;
    }

    if count > 0 {
        hist.iter_mut().for_each(|v| *v /= count as f32);
    }
    hist
}

/// Concatenate MB-LBP histograms at all three scales into one feature vector (768 bins).
pub fn mb_lbp_multi_scale(image: &GrayImage) -> Vec<f32> {
    let h3 = mb_lbp_histogram(image, MbLbpBlockSize::S3);
    let h5 = mb_lbp_histogram(image, MbLbpBlockSize::S5);
    let h9 = mb_lbp_histogram(image, MbLbpBlockSize::S9);
    let mut out = Vec::with_capacity(768);
    out.extend_from_slice(&h3);
    out.extend_from_slice(&h5);
    out.extend_from_slice(&h9);
    out
}

/// Chi-square similarity between two MB-LBP multi-scale histograms.
pub fn mb_lbp_similarity(a: &[f32], b: &[f32]) -> f32 {
    crate::lbp::chi_square_similarity(a, b)
}
