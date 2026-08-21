//! Coloured contact lens detection via iris texture homogeneity.

use eel_abis_iris_segment::{Circle, GrayImage};

/// Estimate the probability that a coloured contact lens is present.
///
/// Coloured lenses produce abnormally uniform iris texture. This function
/// computes the mean absolute deviation (MAD) of pixel values in the iris
/// annulus (between pupil and limbus). A low MAD indicates high homogeneity
/// and therefore a high confidence of a coloured lens.
///
/// Confidence = `1.0 - (mad / 127.5).clamp(0, 1)`.
pub fn detect_coloured(image: &GrayImage, pupil: Circle, limbus: Circle) -> f32 {
    let pixels: Vec<f32> = collect_annulus_pixels(image, &pupil, &limbus);
    if pixels.is_empty() {
        return 0.0;
    }
    let mean = pixels.iter().sum::<f32>() / pixels.len() as f32;
    let mad = pixels.iter().map(|&v| (v - mean).abs()).sum::<f32>() / pixels.len() as f32;
    (1.0 - (mad / 127.5)).clamp(0.0, 1.0)
}

fn collect_annulus_pixels(image: &GrayImage, pupil: &Circle, limbus: &Circle) -> Vec<f32> {
    let x_min = (limbus.cx - limbus.radius).floor().max(0.0) as u32;
    let x_max = (limbus.cx + limbus.radius)
        .ceil()
        .min(image.width as f32 - 1.0) as u32;
    let y_min = (limbus.cy - limbus.radius).floor().max(0.0) as u32;
    let y_max = (limbus.cy + limbus.radius)
        .ceil()
        .min(image.height as f32 - 1.0) as u32;

    let mut pixels = Vec::new();
    for y in y_min..=y_max {
        for x in x_min..=x_max {
            let fx = x as f32;
            let fy = y as f32;
            if limbus.contains(fx, fy) && !pupil.contains(fx, fy) {
                pixels.push(image.get(x, y) as f32);
            }
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_image(w: u32, h: u32, val: u8) -> GrayImage {
        GrayImage::new(w, h, vec![val; (w * h) as usize]).expect("valid image")
    }

    #[test]
    fn uniform_iris_returns_high_confidence() {
        // Perfectly uniform -> MAD = 0 -> confidence = 1.0.
        let image = flat_image(200, 200, 100);
        let pupil = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 20.0,
        };
        let limbus = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 70.0,
        };
        let conf = detect_coloured(&image, pupil, limbus);
        assert!((conf - 1.0).abs() < 1e-5);
    }

    #[test]
    fn confidence_in_range() {
        let image = flat_image(200, 200, 128);
        let pupil = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 20.0,
        };
        let limbus = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 70.0,
        };
        let conf = detect_coloured(&image, pupil, limbus);
        assert!((0.0..=1.0).contains(&conf));
    }

    #[test]
    fn empty_annulus_returns_zero() {
        // Pupil larger than limbus -> no annulus pixels.
        let image = flat_image(200, 200, 128);
        let pupil = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 80.0,
        };
        let limbus = Circle {
            cx: 100.0,
            cy: 100.0,
            radius: 70.0,
        };
        let conf = detect_coloured(&image, pupil, limbus);
        assert_eq!(conf, 0.0);
    }
}
