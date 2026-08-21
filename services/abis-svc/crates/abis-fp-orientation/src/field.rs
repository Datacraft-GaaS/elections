//! Dense orientation field computation.
//!
//! Reference: C. V. K. Rao, "Determining the coherence of local orientations in natural textures,"
//! Computer Vision, Graphics, and Image Processing, 1990. Applied to fingerprints.
//!
//! For each non-overlapping block:
//!   1. Compute Sobel gradients Gx, Gy at each pixel.
//!   2. Accumulate Vx = sum(Gx^2 - Gy^2), Vy = sum(2*Gx*Gy).
//!   3. Dominant orientation theta = 0.5 * atan2(Vy, Vx).
//!   4. Coherence = sqrt(Vx^2+Vy^2) / (Gx^2+Gy^2) in [0,1].

use std::f32::consts::PI;

use crate::{GrayImage, OrientationError};

/// Dense orientation field, one angle per block.
///
/// `angles` and `coherence` are stored row-major with `width` columns and `height` rows
/// (in block units, not pixels).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrientationField {
    /// Orientation angle in [0, PI) radians (π-periodic).
    pub angles: Vec<f32>,
    /// Coherence measure in [0, 255] (255 = fully coherent).
    pub coherence: Vec<u8>,
    /// Number of block columns.
    pub width: u32,
    /// Number of block rows.
    pub height: u32,
    /// Block size in pixels.
    pub block_size: u32,
}

impl OrientationField {
    /// Angle at block (bx, by).  Returns 0.0 for out-of-range.
    pub fn angle_at(&self, bx: i32, by: i32) -> f32 {
        if bx < 0 || by < 0 || bx >= self.width as i32 || by >= self.height as i32 {
            return 0.0;
        }
        self.angles[(by as u32 * self.width + bx as u32) as usize]
    }

    /// Coherence at block (bx, by).  Returns 0 for out-of-range.
    pub fn coherence_at(&self, bx: i32, by: i32) -> u8 {
        if bx < 0 || by < 0 || bx >= self.width as i32 || by >= self.height as i32 {
            return 0;
        }
        self.coherence[(by as u32 * self.width + bx as u32) as usize]
    }
}

/// Compute the orientation field from a grayscale image.
pub fn compute_orientation_field(
    image: &GrayImage,
    block_size: u32,
) -> Result<OrientationField, OrientationError> {
    if image.width < 16 || image.height < 16 {
        return Err(OrientationError::ImageTooSmall);
    }
    let bw = image.width / block_size;
    let bh = image.height / block_size;
    let n = (bw * bh) as usize;
    let mut angles = vec![0.0f32; n];
    let mut coherence = vec![0u8; n];

    for by in 0..bh {
        for bx in 0..bw {
            let px0 = (bx * block_size) as i32;
            let py0 = (by * block_size) as i32;
            let mut vx = 0.0f64;
            let mut vy = 0.0f64;
            let mut energy = 0.0f64;

            for dy in 0..block_size as i32 {
                for dx in 0..block_size as i32 {
                    let gx = image.pixel(px0 + dx + 1, py0 + dy) as f64
                        - image.pixel(px0 + dx - 1, py0 + dy) as f64;
                    let gy = image.pixel(px0 + dx, py0 + dy + 1) as f64
                        - image.pixel(px0 + dx, py0 + dy - 1) as f64;
                    vx += gx * gx - gy * gy;
                    vy += 2.0 * gx * gy;
                    energy += gx * gx + gy * gy;
                }
            }

            let theta = (0.5 * vy.atan2(vx)) as f32;
            // Map to [0, PI).
            let angle = if theta < 0.0 { theta + PI } else { theta };
            let coh = if energy > 1e-6 {
                ((vx * vx + vy * vy).sqrt() / energy * 255.0).min(255.0) as u8
            } else {
                0
            };

            let idx = (by * bw + bx) as usize;
            angles[idx] = angle;
            coherence[idx] = coh;
        }
    }

    Ok(OrientationField {
        angles,
        coherence,
        width: bw,
        height: bh,
        block_size,
    })
}

/// Compute the cross-correlation between two orientation fields after rotation alignment.
///
/// Returns a similarity in [0.0, 1.0].
pub fn field_correlation(
    a: &OrientationField,
    b: &OrientationField,
) -> Result<f32, OrientationError> {
    if a.width != b.width || a.height != b.height {
        return Err(OrientationError::FieldMismatch);
    }
    let n = a.angles.len();
    if n == 0 {
        return Ok(0.0);
    }
    // Orientation correlation: use cos(2*(alpha - beta)) (doubled angles make orientation PI-periodic).
    let corr: f32 = a
        .angles
        .iter()
        .zip(b.angles.iter())
        .map(|(&aa, &ab)| (2.0 * (aa - ab)).cos())
        .sum::<f32>()
        / n as f32;
    // corr in [-1, 1]; map to [0, 1].
    Ok((corr + 1.0) * 0.5)
}
