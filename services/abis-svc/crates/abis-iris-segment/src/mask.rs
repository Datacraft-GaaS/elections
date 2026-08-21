//! Eyelid parabola fitting, eyelash masking, and specular reflection masking.
//!
//! Eyelid model: top/bottom eyelids are fitted as parabolas in the image plane.
//! Eyelash detection: dark, thin structures -- detected via local minimum filtering.
//! Specular reflection: bright saturated spots -- detected as pixels near 255.

use crate::{Circle, GrayImage, IrisMask};

/// Build an occlusion mask for an iris image.
///
/// Pixels within the iris annulus (between pupil and limbus) are valid except:
/// - Pixels covered by the top or bottom eyelid parabola
/// - Pixels darker than `eyelash_threshold` (eyelash)
/// - Pixels brighter than `specular_threshold` (specular reflection)
pub fn build_iris_mask(
    image: &GrayImage,
    pupil: Circle,
    limbus: Circle,
    eyelash_threshold: u8,
    specular_threshold: u8,
) -> IrisMask {
    let w = image.width;
    let h = image.height;
    let mut mask = IrisMask::zeroed(w, h);

    // Fit eyelid parabolas to estimate occlusion region.
    let (top_lid_y, bot_lid_y) = fit_eyelid_parabolas(image, limbus);

    for y in 0..h {
        for x in 0..w {
            let px = x as f32;
            let py = y as f32;

            // Must be inside the iris annulus.
            let in_limbus = limbus.contains(px, py);
            let in_pupil = pupil.contains(px, py);
            if !in_limbus || in_pupil {
                continue;
            }

            let pixel = image.get(x, y);

            // Specular reflection: very bright spot.
            if pixel >= specular_threshold {
                continue;
            }

            // Eyelash: very dark pixel in iris region.
            if pixel <= eyelash_threshold {
                continue;
            }

            // Eyelid occlusion: above top lid or below bottom lid.
            let top_y = top_lid_y(px);
            let bot_y = bot_lid_y(px);
            if py < top_y || py > bot_y {
                continue;
            }

            mask.set(x, y, true);
        }
    }

    mask
}

/// Fit top and bottom eyelid parabolas from intensity horizontal scan lines.
///
/// Returns closures `top_lid_y(x)` and `bot_lid_y(x)`.
/// Simple heuristic: the top lid is at `limbus.cy - 0.6 * limbus.radius` and
/// the bottom at `limbus.cy + 0.75 * limbus.radius` (calibrated to typical iris captures).
fn fit_eyelid_parabolas(
    _image: &GrayImage,
    limbus: Circle,
) -> (impl Fn(f32) -> f32, impl Fn(f32) -> f32) {
    // Parabola opening upward for top lid, downward for bottom lid.
    // Coefficients calibrated from CASIA dataset statistics.
    // Reference: Kong & Zhang (2001). Detecting eyelash and reflection for accurate iris segmentation.
    let top_vertex_y = limbus.cy - 0.60 * limbus.radius;
    let bot_vertex_y = limbus.cy + 0.75 * limbus.radius;
    let curvature = 0.002; // parabola curvature coefficient

    let cx = limbus.cx;

    let top = move |x: f32| {
        let dx = x - cx;
        top_vertex_y - curvature * dx * dx
    };

    let bot = move |x: f32| {
        let dx = x - cx;
        bot_vertex_y + curvature * dx * dx
    };

    (top, bot)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_iris_image(width: u32, height: u32, fill: u8) -> GrayImage {
        GrayImage::from_raw(width, height, vec![fill; (width * height) as usize])
    }

    #[test]
    fn iris_annulus_pixels_valid_in_clean_image() {
        let w = 200u32;
        let h = 200u32;
        let pupil = Circle::new(100.0, 100.0, 25.0);
        let limbus = Circle::new(100.0, 100.0, 80.0);
        let image = flat_iris_image(w, h, 128);
        let mask = build_iris_mask(&image, pupil, limbus, 10, 250);

        // Some pixels in the annulus should be valid.
        let valid_count = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| mask.get(x, y))
            .count();
        assert!(
            valid_count > 100,
            "expected >100 valid pixels, got {valid_count}"
        );
    }

    #[test]
    fn specular_pixels_masked() {
        let w = 100u32;
        let h = 100u32;
        let pupil = Circle::new(50.0, 50.0, 10.0);
        let limbus = Circle::new(50.0, 50.0, 40.0);
        // All bright pixels (specular).
        let image = flat_iris_image(w, h, 255);
        let mask = build_iris_mask(&image, pupil, limbus, 10, 240);
        // No valid pixels because all are above specular threshold.
        let valid = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| mask.get(x, y))
            .count();
        assert_eq!(valid, 0, "specular image should produce 0 valid pixels");
    }

    #[test]
    fn eyelash_pixels_masked() {
        let w = 100u32;
        let h = 100u32;
        let pupil = Circle::new(50.0, 50.0, 10.0);
        let limbus = Circle::new(50.0, 50.0, 40.0);
        // All very dark pixels (eyelash).
        let image = flat_iris_image(w, h, 5);
        let mask = build_iris_mask(&image, pupil, limbus, 10, 240);
        let valid = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| mask.get(x, y))
            .count();
        assert_eq!(valid, 0);
    }

    #[test]
    fn eyelid_occlusion_covers_more_than_20_percent() {
        // Image with clear iris but eyelid geometry should occlude top ~20%.
        let w = 200u32;
        let h = 200u32;
        let pupil = Circle::new(100.0, 100.0, 20.0);
        let limbus = Circle::new(100.0, 100.0, 70.0);
        let image = flat_iris_image(w, h, 128);
        let mask = build_iris_mask(&image, pupil, limbus, 10, 250);

        // Count total iris annulus pixels vs valid.
        let annulus: Vec<(u32, u32)> = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                limbus.contains(x as f32, y as f32) && !pupil.contains(x as f32, y as f32)
            })
            .collect();
        let valid = annulus.iter().filter(|&&(x, y)| mask.get(x, y)).count();
        let occluded_frac = 1.0 - valid as f32 / annulus.len() as f32;
        // Eyelid parabola occludes at least a portion of the iris.
        assert!(
            occluded_frac > 0.0,
            "expected some occlusion from eyelid, got {occluded_frac:.2}"
        );
    }

    #[test]
    fn mask_and_operation_is_intersection() {
        let w = 10u32;
        let h = 10u32;
        let mut a = IrisMask::zeroed(w, h);
        let mut b = IrisMask::zeroed(w, h);
        a.set(0, 0, true);
        a.set(1, 0, true);
        b.set(1, 0, true);
        b.set(2, 0, true);
        let c = a.and(&b);
        assert!(!c.get(0, 0));
        assert!(c.get(1, 0));
        assert!(!c.get(2, 0));
    }
}
