//! FingerCode implementation.
//!
//! Reference: A. K. Jain, S. Prabhakar, L. Hong, S. Pankanti,
//! "Filterbank-based fingerprint matching," IEEE Trans. Image Process.,
//! vol. 9, no. 5, pp. 846-859, May 2000.
//!
//! The algorithm:
//! 1. Locate the reference point (core) via the Poincare index on the local orientation field.
//! 2. Tessellate the image into 80 sectors (5 concentric bands x 16 angular sectors).
//! 3. Apply a bank of 8 Gabor filters (orientations 0..157.5 deg in 22.5 deg steps).
//! 4. For each sector and each filtered image, compute the mean absolute deviation (MAD) of pixel
//!    values -- gives one scalar per (sector, filter) pair.
//! 5. The FingerCode feature vector has 80 * 8 = 640 elements.

use std::f32::consts::PI;

use crate::{GrayImage, Point, TextureError};

// Number of angular sectors per band.
const N_ANGULAR: usize = 16;
// Number of radial bands.
const N_BANDS: usize = 5;
// Total sectors.
const N_SECTORS: usize = N_ANGULAR * N_BANDS;
// Number of Gabor filter orientations.
const N_FILTERS: usize = 8;
// Feature vector length.
pub const FINGERCODE_LEN: usize = N_SECTORS * N_FILTERS;

// Inner and outer radii for each band (in pixels, assuming ~500 ppi image, 300x300 crop).
const BAND_RADII: [(f32, f32); N_BANDS] = [
    (4.0, 25.0),
    (25.0, 50.0),
    (50.0, 75.0),
    (75.0, 100.0),
    (100.0, 125.0),
];

/// A single filtered image produced by one Gabor filter.
#[derive(Debug, Clone)]
pub struct FilteredImage {
    pub pixels: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

impl FilteredImage {
    fn pixel(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 0.0;
        }
        self.pixels[(y as u32 * self.width + x as u32) as usize]
    }
}

/// One sector -- holds pixel indices within it.
pub struct Sector {
    pub indices: Vec<(u32, u32)>,
}

/// The FingerCode feature vector (640-dimensional).
///
/// Stored as Vec<f32> for serde compatibility (large fixed-size arrays require
/// specialised serde support).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FingerCode {
    pub features: Vec<f32>,
}

impl FingerCode {
    fn zero() -> Self {
        Self {
            features: vec![0.0f32; FINGERCODE_LEN],
        }
    }
}

/// Locate the reference point (core) using a simplified Poincare index on the local orientation
/// field.
///
/// The orientation field is estimated from image gradients over non-overlapping blocks.
/// A core is identified where the Poincare index rounds to +1.
pub fn locate_reference_point(image: &GrayImage) -> Result<Point, TextureError> {
    if image.width < 16 || image.height < 16 {
        return Err(TextureError::ImageTooSmall);
    }

    let block = 8u32;
    let cols = (image.width / block) as i32;
    let rows = (image.height / block) as i32;

    // Compute local orientation angles for each block.
    let mut angles: Vec<f32> = vec![0.0; (cols * rows) as usize];
    for by in 0..rows {
        for bx in 0..cols {
            let angle = block_orientation(image, bx as u32 * block, by as u32 * block, block);
            angles[(by * cols + bx) as usize] = angle;
        }
    }

    // Compute Poincare index for each interior block.
    let mut best_score = -1.0f32;
    let mut best = Point::new(image.width as f32 / 2.0, image.height as f32 / 2.0);
    for by in 1..rows - 1 {
        for bx in 1..cols - 1 {
            // 8-neighbour clockwise loop.
            let neighbours = [
                angles[((by - 1) * cols + (bx - 1)) as usize],
                angles[((by - 1) * cols + bx) as usize],
                angles[((by - 1) * cols + (bx + 1)) as usize],
                angles[(by * cols + (bx + 1)) as usize],
                angles[((by + 1) * cols + (bx + 1)) as usize],
                angles[((by + 1) * cols + bx) as usize],
                angles[((by + 1) * cols + (bx - 1)) as usize],
                angles[(by * cols + (bx - 1)) as usize],
            ];
            let pi = poincare_index(&neighbours);
            // Core has Poincare index close to 0.5 (in units of pi).
            let score = 1.0 - (pi - 0.5).abs();
            if score > best_score {
                best_score = score;
                best = Point::new(
                    (bx as f32 + 0.5) * block as f32,
                    (by as f32 + 0.5) * block as f32,
                );
            }
        }
    }

    if best_score <= 0.5 {
        // No block scored above the all-zero baseline (0.5 for pi=0).
        // Fall back to image centre -- common for arch patterns and flat inputs.
        Ok(Point::new(
            image.width as f32 / 2.0,
            image.height as f32 / 2.0,
        ))
    } else {
        Ok(best)
    }
}

/// Compute orientation angle (in radians) of a block using gradient method (Rao 1990).
fn block_orientation(image: &GrayImage, bx: u32, by: u32, block: u32) -> f32 {
    let mut gxx = 0.0f64;
    let mut gyy = 0.0f64;
    let mut gxy = 0.0f64;
    for dy in 0..block {
        for dx in 0..block {
            let x = (bx + dx) as i32;
            let y = (by + dy) as i32;
            let gx = image.pixel(x + 1, y) as f64 - image.pixel(x - 1, y) as f64;
            let gy = image.pixel(x, y + 1) as f64 - image.pixel(x, y - 1) as f64;
            gxx += gx * gx;
            gyy += gy * gy;
            gxy += gx * gy;
        }
    }
    (0.5 * (2.0 * gxy).atan2(gxx - gyy)) as f32
}

/// Compute the Poincare index for a sequence of 8 neighbourhood orientation angles.
/// Returns a value near 0.5 for a core, near -0.5 for a delta, near 0 otherwise.
fn poincare_index(angles: &[f32; 8]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..8 {
        let diff = angle_diff(angles[(i + 1) % 8], angles[i]);
        sum += diff;
    }
    sum / (2.0 * PI)
}

/// Oriented-angle difference in [-PI/2, PI/2] (orientation, not direction).
fn angle_diff(a: f32, b: f32) -> f32 {
    let mut d = a - b;
    while d > PI / 2.0 {
        d -= PI;
    }
    while d < -PI / 2.0 {
        d += PI;
    }
    d
}

/// Tessellate the image into sectors around the reference point.
///
/// Produces N_BANDS * N_ANGULAR = 80 sectors, arranged as concentric annular bands each
/// divided into 16 equal angular slices.
pub fn tessellate(image: &GrayImage, ref_point: Point) -> Vec<Sector> {
    let mut sectors: Vec<Sector> = (0..N_SECTORS)
        .map(|_| Sector {
            indices: Vec::new(),
        })
        .collect();

    let cx = ref_point.x;
    let cy = ref_point.y;
    let outer_r = BAND_RADII[N_BANDS - 1].1;

    let x_min = ((cx - outer_r).max(0.0) as u32).min(image.width.saturating_sub(1));
    let x_max = ((cx + outer_r) as u32 + 1).min(image.width);
    let y_min = ((cy - outer_r).max(0.0) as u32).min(image.height.saturating_sub(1));
    let y_max = ((cy + outer_r) as u32 + 1).min(image.height);

    for py in y_min..y_max {
        for px in x_min..x_max {
            let dx = px as f32 - cx;
            let dy = py as f32 - cy;
            let r = (dx * dx + dy * dy).sqrt();
            // Find which band this pixel belongs to.
            let band = BAND_RADII
                .iter()
                .position(|&(inner, outer)| r >= inner && r < outer);
            if let Some(b) = band {
                // Angular sector: atan2 mapped to [0, 2*PI), divided into N_ANGULAR slices.
                let angle = (dy.atan2(dx) + 2.0 * PI) % (2.0 * PI);
                let sector_idx = (angle / (2.0 * PI) * N_ANGULAR as f32) as usize % N_ANGULAR;
                let flat_idx = b * N_ANGULAR + sector_idx;
                sectors[flat_idx].indices.push((px, py));
            }
        }
    }

    sectors
}

/// Apply a bank of 8 Gabor filters at orientations 0, 22.5, 45, 67.5, 90, 112.5, 135, 157.5 deg.
///
/// Each filter is a 2-D Gabor with spatial frequency matching typical ridge period (~500 ppi).
/// Reference: Jain et al. 2000, Section III-B.
pub fn filter_bank(image: &GrayImage, _orientations: &[u8]) -> Vec<FilteredImage> {
    let freq = 1.0 / 10.0f64; // ridge frequency ~10px period at 500 ppi
    let sigma = 4.0f64;
    (0..N_FILTERS)
        .map(|k| {
            let theta = k as f64 * PI as f64 / N_FILTERS as f64;
            gabor_filter(image, freq, theta, sigma, sigma)
        })
        .collect()
}

/// Apply a single Gabor filter and return the filtered image as f32 values.
fn gabor_filter(
    image: &GrayImage,
    frequency: f64,
    orientation: f64,
    sigma_x: f64,
    sigma_y: f64,
) -> FilteredImage {
    const HALF: i32 = 7;
    const KSIZE: i32 = 2 * HALF + 1;
    let cos_t = orientation.cos();
    let sin_t = orientation.sin();
    let w = image.width;
    let h = image.height;
    let mut out = vec![0.0f32; (w * h) as usize];

    // Precompute kernel.
    let mut kernel = [[0.0f64; 15]; 15];
    for ky in -HALF..=HALF {
        for kx in -HALF..=HALF {
            let xr = kx as f64 * cos_t + ky as f64 * sin_t;
            let yr = -kx as f64 * sin_t + ky as f64 * cos_t;
            let gauss =
                (-0.5 * (xr * xr / (sigma_x * sigma_x) + yr * yr / (sigma_y * sigma_y))).exp();
            let wave = (2.0 * std::f64::consts::PI * frequency * xr).cos();
            kernel[(ky + HALF) as usize][(kx + HALF) as usize] = gauss * wave;
        }
    }

    for py in 0..h as i32 {
        for px in 0..w as i32 {
            let mut val = 0.0f64;
            for ky in -HALF..=HALF {
                for kx in -HALF..=HALF {
                    val += image.pixel(px + kx, py + ky) as f64
                        * kernel[(ky + HALF) as usize][(kx + HALF) as usize];
                }
            }
            out[(py as u32 * w + px as u32) as usize] = val.abs() as f32;
        }
    }

    let _ = KSIZE; // used implicitly via HALF
    FilteredImage {
        pixels: out,
        width: w,
        height: h,
    }
}

/// Encode the FingerCode: for each (sector, filter) pair compute the mean absolute deviation.
///
/// Feature index layout: sector_idx * N_FILTERS + filter_idx.
pub fn encode_fingercode(sectors: &[Sector], filtered: &[FilteredImage]) -> FingerCode {
    let mut fc = FingerCode::zero();
    for (si, sector) in sectors.iter().enumerate().take(N_SECTORS) {
        for (fi, fimg) in filtered.iter().enumerate().take(N_FILTERS) {
            let feat_idx = si * N_FILTERS + fi;
            if sector.indices.is_empty() {
                fc.features[feat_idx] = 0.0;
                continue;
            }
            // Compute mean.
            let sum: f32 = sector
                .indices
                .iter()
                .map(|&(x, y)| fimg.pixel(x as i32, y as i32))
                .sum();
            let mean = sum / sector.indices.len() as f32;
            // Compute MAD (mean absolute deviation).
            let mad: f32 = sector
                .indices
                .iter()
                .map(|&(x, y)| (fimg.pixel(x as i32, y as i32) - mean).abs())
                .sum::<f32>()
                / sector.indices.len() as f32;
            fc.features[feat_idx] = mad;
        }
    }
    fc
}

/// Match two FingerCode vectors using normalized Euclidean distance.
///
/// Returns a similarity score in [0.0, 1.0] where 1.0 = identical.
/// Distance is normalized against a reference distance of 100.0 (empirical).
pub fn match_fingercode(a: &FingerCode, b: &FingerCode) -> f32 {
    let dist_sq: f32 = a
        .features
        .iter()
        .zip(b.features.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum();
    let dist = dist_sq.sqrt();
    // Normalise: at distance >= 100 we consider a non-match.
    let similarity = 1.0 - (dist / 100.0).min(1.0);
    similarity.max(0.0)
}
