//! Daugman rubber-sheet (polar unwrapping) normalisation.
//!
//! Maps the iris annulus from Cartesian to a fixed-size polar grid.
//! Each row of the output corresponds to a fractional radius rho in [0, 1]
//! (0 = pupil boundary, 1 = limbus boundary); each column is an angle theta in [0, 2*pi).
//!
//! Reference: Daugman, J. (2004). How iris recognition works. IEEE TCSV, 14(1), 21-30.

use crate::{NORM_HEIGHT, NORM_WIDTH};
use eel_abis_iris_segment::{Circle, GrayImage, IrisSegmentation};

/// Normalised 512x64 iris image (polar unwrapped).
#[derive(Debug, Clone)]
pub struct NormalisedIris {
    pub data: Vec<u8>,
}

impl NormalisedIris {
    /// Width of the normalised image (angular axis).
    pub const WIDTH: u32 = NORM_WIDTH;
    /// Height of the normalised image (radial axis).
    pub const HEIGHT: u32 = NORM_HEIGHT;

    /// Pixel at column `col` (angle) and row `row` (radius).
    pub fn get(&self, col: u32, row: u32) -> u8 {
        self.data[(row * NORM_WIDTH + col) as usize]
    }
}

/// Apply rubber-sheet transform to produce a 512x64 normalised iris image.
pub fn rubber_sheet(image: &GrayImage, seg: &IrisSegmentation) -> NormalisedIris {
    let pupil = seg.pupil;
    let limbus = seg.limbus;
    let w = NORM_WIDTH;
    let h = NORM_HEIGHT;
    let mut data = vec![0u8; (w * h) as usize];

    for row in 0..h {
        let rho = row as f32 / (h - 1) as f32; // [0, 1]: pupil -> limbus
        for col in 0..w {
            let theta = 2.0 * std::f32::consts::PI * col as f32 / w as f32;
            let (px, py) = interpolate_point(pupil, limbus, rho, theta);
            let ix = px.round() as u32;
            let iy = py.round() as u32;
            data[(row * w + col) as usize] = image.get(ix, iy);
        }
    }

    NormalisedIris { data }
}

/// Interpolate a point at fractional radius `rho` and angle `theta` between pupil and limbus.
///
/// Uses the Daugman homogeneous rubber-sheet model: both boundaries are circles,
/// so the x/y coordinates are linearly interpolated between the pupil and limbus boundary points.
fn interpolate_point(pupil: Circle, limbus: Circle, rho: f32, theta: f32) -> (f32, f32) {
    let cos_t = theta.cos();
    let sin_t = theta.sin();

    let px = pupil.cx + pupil.radius * cos_t;
    let py = pupil.cy + pupil.radius * sin_t;
    let lx = limbus.cx + limbus.radius * cos_t;
    let ly = limbus.cy + limbus.radius * sin_t;

    let x = (1.0 - rho) * px + rho * lx;
    let y = (1.0 - rho) * py + rho * ly;
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_segment::{IrisMask, SegmentIris};

    fn make_seg(width: u32, height: u32, pr: f32, lr: f32) -> (GrayImage, IrisSegmentation) {
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let data: Vec<u8> = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
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
        let image = GrayImage::from_raw(width, height, data.clone());
        let pupil = Circle::new(cx, cy, pr);
        let limbus = Circle::new(cx, cy, lr);
        let mask = IrisMask::all_valid(width, height);
        let seg = IrisSegmentation {
            pupil,
            limbus,
            mask,
            iris_sclera_ratio: 0.9,
        };
        (image, seg)
    }

    #[test]
    fn normalised_output_is_512x64() {
        let (img, seg) = make_seg(240, 240, 25.0, 80.0);
        let norm = rubber_sheet(&img, &seg);
        assert_eq!(norm.data.len(), (NORM_WIDTH * NORM_HEIGHT) as usize);
    }

    #[test]
    fn normalised_pixel_values_in_range() {
        let (img, seg) = make_seg(240, 240, 25.0, 80.0);
        let norm = rubber_sheet(&img, &seg);
        for &p in &norm.data {
            assert!(p <= 255);
        }
    }

    #[test]
    fn same_image_same_normalisation() {
        let (img, seg) = make_seg(240, 240, 25.0, 80.0);
        let n1 = rubber_sheet(&img, &seg);
        let n2 = rubber_sheet(&img, &seg);
        assert_eq!(n1.data, n2.data);
    }

    #[test]
    fn interpolate_at_rho_zero_is_on_pupil() {
        let pupil = Circle::new(100.0, 100.0, 30.0);
        let limbus = Circle::new(100.0, 100.0, 90.0);
        let (x, y) = interpolate_point(pupil, limbus, 0.0, 0.0);
        assert!((x - 130.0).abs() < 0.01, "expected 130.0, got {x}");
        assert!((y - 100.0).abs() < 0.01, "expected 100.0, got {y}");
    }

    #[test]
    fn interpolate_at_rho_one_is_on_limbus() {
        let pupil = Circle::new(100.0, 100.0, 30.0);
        let limbus = Circle::new(100.0, 100.0, 90.0);
        let (x, y) = interpolate_point(pupil, limbus, 1.0, 0.0);
        assert!((x - 190.0).abs() < 0.01, "expected 190.0, got {x}");
        assert!((y - 100.0).abs() < 0.01, "expected 100.0, got {y}");
    }
}
