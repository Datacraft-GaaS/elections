//! Transparent contact lens detection via specular ring analysis.

use eel_abis_iris_segment::{Circle, GrayImage};

const ANGULAR_SAMPLES: u32 = 64;
const BRIGHT_THRESHOLD: u8 = 200;
/// Scale factor relative to limbus radius where the contact lens edge appears.
const LENS_EDGE_SCALE: f32 = 1.05;

/// Estimate the probability that a transparent contact lens is present.
///
/// Samples 64 angular positions at ~1.05x the limbus radius and counts the
/// fraction where pixel brightness exceeds 200 (specular reflection from the
/// lens edge). Returns a confidence in [0, 1].
pub fn detect_transparent(image: &GrayImage, limbus: Circle) -> f32 {
    let ring_r = limbus.radius * LENS_EDGE_SCALE;
    let mut bright_count = 0u32;

    for i in 0..ANGULAR_SAMPLES {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (ANGULAR_SAMPLES as f32);
        let x = limbus.cx + ring_r * angle.cos();
        let y = limbus.cy + ring_r * angle.sin();
        // Clamp to image bounds.
        let px = x.round().clamp(0.0, image.width.saturating_sub(1) as f32) as u32;
        let py = y.round().clamp(0.0, image.height.saturating_sub(1) as f32) as u32;
        if image.get(px, py) > BRIGHT_THRESHOLD {
            bright_count += 1;
        }
    }

    bright_count as f32 / ANGULAR_SAMPLES as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_image(w: u32, h: u32, val: u8) -> GrayImage {
        GrayImage::new(w, h, vec![val; (w * h) as usize]).expect("valid image")
    }

    #[test]
    fn dark_image_returns_low_confidence() {
        let image = flat_image(200, 200, 50);
        let limbus = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 60.0,
        };
        let conf = detect_transparent(&image, limbus);
        assert_eq!(conf, 0.0);
    }

    #[test]
    fn bright_ring_returns_high_confidence() {
        // Fully bright image: all samples exceed threshold.
        let image = flat_image(400, 400, 255);
        let limbus = Circle {
            cx: 200.0,
            cy: 200.0,
            radius: 80.0,
        };
        let conf = detect_transparent(&image, limbus);
        assert!((conf - 1.0).abs() < 1e-5);
    }

    #[test]
    fn confidence_in_range() {
        let image = flat_image(200, 200, 128);
        let limbus = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 50.0,
        };
        let conf = detect_transparent(&image, limbus);
        assert!((0.0..=1.0).contains(&conf));
    }
}
