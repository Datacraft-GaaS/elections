//! Slap segmentation: extract individual finger images from a rolled/slapped capture.
//!
//! Reference: ISO/IEC 29109-2:2018, Section 5 -- slap segmentation requirements.
//!
//! Algorithm:
//! 1. Otsu binarisation to separate foreground (skin) from background.
//! 2. Horizontal projection profile to find valleys between fingers.
//! 3. Assign positions by x-coordinate order and slap type.

use crate::{GrayImage, MultiFingerError};

/// ISO/IEC 19794-2 finger positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum FingerPosition {
    RightThumb = 1,
    RightIndex = 2,
    RightMiddle = 3,
    RightRing = 4,
    RightLittle = 5,
    LeftThumb = 6,
    LeftIndex = 7,
    LeftMiddle = 8,
    LeftRing = 9,
    LeftLittle = 10,
}

/// Which slap the image represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlapKind {
    RightFour,
    LeftFour,
    Thumbs,
}

impl SlapKind {
    /// Expected number of fingers.
    pub fn finger_count(self) -> usize {
        match self {
            SlapKind::RightFour | SlapKind::LeftFour => 4,
            SlapKind::Thumbs => 2,
        }
    }

    /// Ordered finger positions (left-to-right in the capture).
    pub fn positions(self) -> &'static [FingerPosition] {
        match self {
            SlapKind::RightFour => &[
                FingerPosition::RightIndex,
                FingerPosition::RightMiddle,
                FingerPosition::RightRing,
                FingerPosition::RightLittle,
            ],
            SlapKind::LeftFour => &[
                FingerPosition::LeftLittle,
                FingerPosition::LeftRing,
                FingerPosition::LeftMiddle,
                FingerPosition::LeftIndex,
            ],
            SlapKind::Thumbs => &[FingerPosition::RightThumb, FingerPosition::LeftThumb],
        }
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Compute Otsu's threshold for a grayscale pixel slice.
pub fn otsu_threshold(pixels: &[u8]) -> u8 {
    let mut hist = [0u64; 256];
    for &p in pixels {
        hist[p as usize] += 1;
    }
    let total = pixels.len() as f64;
    let mut sum_total = 0.0f64;
    for (i, &h) in hist.iter().enumerate() {
        sum_total += i as f64 * h as f64;
    }
    let mut sum_bg = 0.0f64;
    let mut w_bg = 0u64;
    let mut best_var = 0.0f64;
    let mut best_t = 128u8;
    for t in 0..256usize {
        w_bg += hist[t];
        if w_bg == 0 {
            continue;
        }
        let w_fg = total as u64 - w_bg;
        if w_fg == 0 {
            break;
        }
        sum_bg += t as f64 * hist[t] as f64;
        let mean_bg = sum_bg / w_bg as f64;
        let mean_fg = (sum_total - sum_bg) / w_fg as f64;
        let var = w_bg as f64 * w_fg as f64 * (mean_bg - mean_fg).powi(2);
        if var > best_var {
            best_var = var;
            best_t = t as u8;
        }
    }
    best_t
}

/// Binarise the image using Otsu's method; 255 = foreground.
fn binarise(image: &GrayImage) -> Vec<u8> {
    let t = otsu_threshold(&image.pixels);
    image
        .pixels
        .iter()
        .map(|&p| if p <= t { 255u8 } else { 0u8 })
        .collect()
}

/// Vertical projection profile: count foreground pixels in each column.
fn vertical_projection(binary: &[u8], width: u32, height: u32) -> Vec<u32> {
    let mut proj = vec![0u32; width as usize];
    for y in 0..height {
        for x in 0..width {
            if binary[(y * width + x) as usize] > 0 {
                proj[x as usize] += 1;
            }
        }
    }
    proj
}

/// Find valley positions (column indices with projection below threshold).
fn find_valleys(proj: &[u32], threshold: u32, min_valley_width: usize) -> Vec<(usize, usize)> {
    let mut valleys: Vec<(usize, usize)> = Vec::new();
    let mut in_valley = false;
    let mut vstart = 0;
    for (x, &count) in proj.iter().enumerate() {
        if count <= threshold {
            if !in_valley {
                in_valley = true;
                vstart = x;
            }
        } else if in_valley {
            let vend = x;
            if vend - vstart >= min_valley_width {
                valleys.push((vstart, vend));
            }
            in_valley = false;
        }
    }
    if in_valley && proj.len() - vstart >= min_valley_width {
        valleys.push((vstart, proj.len()));
    }
    valleys
}

/// Segment a slap image into individual finger images.
///
/// Returns a vec of (FingerPosition, BoundingBox, cropped GrayImage) in left-to-right order.
pub fn segment_slap(
    image: &GrayImage,
    kind: SlapKind,
) -> Result<Vec<(FingerPosition, BoundingBox, GrayImage)>, MultiFingerError> {
    if image.width < 16 || image.height < 16 {
        return Err(MultiFingerError::SegmentationFailed(
            "image too small".into(),
        ));
    }

    let binary = binarise(image);
    let proj = vertical_projection(&binary, image.width, image.height);

    // Threshold: 10% of image height = a finger column.
    let proj_threshold = image.height / 10;
    // Minimum valley width: 1/16 of image width.
    let min_valley_width = (image.width / 16).max(2) as usize;

    let valleys = find_valleys(&proj, proj_threshold, min_valley_width);

    // Build finger column ranges from valleys.
    let n_fingers = kind.finger_count();
    let mut boundaries: Vec<u32> = vec![0];
    for (vs, ve) in &valleys {
        let mid = (*vs + *ve) / 2;
        boundaries.push(mid as u32);
    }
    boundaries.push(image.width);

    // We need exactly n_fingers segments.
    let segments = if boundaries.len() >= n_fingers + 1 {
        // Use the first n_fingers+1 boundaries.
        boundaries.truncate(n_fingers + 1);
        boundaries
    } else {
        // Not enough valleys detected -- divide evenly.
        let step = image.width / n_fingers as u32;
        (0..=n_fingers).map(|i| i as u32 * step).collect()
    };

    let positions = kind.positions();
    let mut result = Vec::with_capacity(n_fingers);

    for i in 0..n_fingers {
        let x0 = segments[i];
        let x1 = segments[i + 1].min(image.width);
        let w = x1.saturating_sub(x0).max(1);
        let bb = BoundingBox {
            x: x0,
            y: 0,
            width: w,
            height: image.height,
        };
        let crop = image.crop(&bb);
        result.push((positions[i], bb, crop));
    }

    Ok(result)
}
