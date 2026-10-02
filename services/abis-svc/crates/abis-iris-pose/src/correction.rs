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

//! Off-axis pixel correction: translates an iris image to centre the pupil on the limbus.

use eel_abis_iris_encode::IrisTemplate;
use eel_abis_iris_segment::{GrayImage, IrisSegmentation};

/// Corrects an iris image for off-axis gaze by shifting pixels.
pub struct OffAxisCorrector;

impl OffAxisCorrector {
    /// Translate the image so the pupil centre aligns with the limbus centre.
    ///
    /// Each output pixel at (x, y) samples the input at (x + dx, y + dy) where
    /// (dx, dy) is the pupil-to-limbus displacement. Pixels sourced outside the
    /// image boundary are set to 0.
    pub fn correct(image: &GrayImage, seg: &IrisSegmentation) -> GrayImage {
        let dx = (seg.pupil.cx - seg.limbus.cx).round() as i64;
        let dy = (seg.pupil.cy - seg.limbus.cy).round() as i64;
        let w = image.width;
        let h = image.height;

        let data: Vec<u8> = (0..h)
            .flat_map(|y| {
                (0..w).map(move |x| {
                    let sx = x as i64 + dx;
                    let sy = y as i64 + dy;
                    if sx >= 0 && sx < w as i64 && sy >= 0 && sy < h as i64 {
                        image.get(sx as u32, sy as u32)
                    } else {
                        0
                    }
                })
            })
            .collect();

        GrayImage::new(w, h, data).expect("corrected image dimensions must match")
    }

    /// Returns true when the corrected template has a lower Hamming distance to
    /// the reference than the acceptance threshold of 0.40.
    pub fn reduces_hd(corrected_template: &IrisTemplate, reference: &IrisTemplate) -> bool {
        corrected_template
            .hamming_distance(reference, 0.3)
            .map(|hd| hd < 0.40)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_segment::{Circle, IrisMask, IrisSegmentation};

    fn flat_image(w: u32, h: u32, val: u8) -> GrayImage {
        GrayImage::new(w, h, vec![val; (w * h) as usize]).expect("valid image")
    }

    fn make_seg(px: f32, py: f32, lx: f32, ly: f32, r: f32) -> IrisSegmentation {
        IrisSegmentation {
            pupil: Circle {
                cx: px,
                cy: py,
                radius: r * 0.4,
            },
            limbus: Circle {
                cx: lx,
                cy: ly,
                radius: r,
            },
            mask: IrisMask::all_valid(100, 100),
            iris_sclera_ratio: 0.5,
        }
    }

    #[test]
    fn correct_returns_same_size() {
        let image = flat_image(80, 60, 128);
        let seg = make_seg(45.0, 35.0, 40.0, 30.0, 30.0);
        let out = OffAxisCorrector::correct(&image, &seg);
        assert_eq!(out.width, 80);
        assert_eq!(out.height, 60);
    }

    #[test]
    fn correct_identity_when_frontal() {
        // Build a non-uniform image so we can detect if it was altered.
        let mut data = vec![0u8; 100 * 100];
        for i in 0..100u32 {
            for j in 0..100u32 {
                data[(i * 100 + j) as usize] = ((i + j) % 256) as u8;
            }
        }
        let image = GrayImage::new(100, 100, data.clone()).expect("valid");
        // Pupil centred on limbus: no displacement.
        let seg = make_seg(50.0, 50.0, 50.0, 50.0, 40.0);
        let out = OffAxisCorrector::correct(&image, &seg);
        // Interior pixels (clear of boundary) must be unchanged.
        for x in 1..99u32 {
            assert_eq!(out.get(x, 50), image.get(x, 50));
        }
    }

    #[test]
    fn correct_shifts_pixels_for_off_axis_input() {
        // Place a bright spot at (50, 50).
        let mut data = vec![0u8; 100 * 100];
        data[50 * 100 + 50] = 255;
        let image = GrayImage::new(100, 100, data).expect("valid");

        // Pupil displaced 5px right of limbus: dx=+5, dy=0.
        // Source for output pixel (x,y) is (x+5, y).
        // The bright spot at source (50,50) -> output (45,50).
        let seg = make_seg(55.0, 50.0, 50.0, 50.0, 40.0);
        let out = OffAxisCorrector::correct(&image, &seg);
        assert_eq!(out.get(45, 50), 255);
        assert_eq!(out.get(50, 50), 0);
    }

    #[test]
    fn correct_does_not_panic_on_edge_pixels() {
        let image = flat_image(10, 10, 100);
        // Large off-axis: most output pixels will be out-of-bounds (zero).
        let seg = make_seg(18.0, 18.0, 5.0, 5.0, 4.0);
        let out = OffAxisCorrector::correct(&image, &seg);
        assert_eq!(out.width, 10);
        assert_eq!(out.height, 10);
    }

    #[test]
    fn correct_out_of_bounds_pixels_are_zero() {
        let image = flat_image(20, 20, 200);
        // Displace well outside image bounds.
        let seg = make_seg(50.0, 50.0, 10.0, 10.0, 8.0);
        let out = OffAxisCorrector::correct(&image, &seg);
        // All source coords are at x+40, y+40 which is >= 20 => all zero.
        for x in 0..20u32 {
            for y in 0..20u32 {
                assert_eq!(out.get(x, y), 0);
            }
        }
    }

    #[test]
    fn reduces_hd_returns_bool() {
        use eel_abis_iris_encode::daugman_code::DaugmanCode;
        use eel_abis_iris_encode::pie::PieCode;
        use eel_abis_iris_encode::{IrisTemplate, IRIS_CODE_BYTES};

        // Two identical templates: HD = 0.0, which is < 0.40.
        let code = [0xAAu8; IRIS_CODE_BYTES];
        let mask = [0xFFu8; IRIS_CODE_BYTES];
        let daugman = DaugmanCode { code, mask };
        let pie = PieCode { code, mask };
        let t1 = IrisTemplate {
            daugman: daugman.clone(),
            pie: pie.clone(),
            quality: 0.9,
        };
        let t2 = IrisTemplate {
            daugman,
            pie,
            quality: 0.9,
        };
        let result = OffAxisCorrector::reduces_hd(&t1, &t2);
        assert!(result);
    }
}
