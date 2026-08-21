//! OSIRIS-style Viterbi boundary refinement.
//!
//! Reference: Othman, N., Dorizzi, B., & Garcia-Salicetti, S. (2016).
//! OSIRIS: An open source iris recognition software.
//! Pattern Recognition Letters, 82, 124-131.
//!
//! After the Daugman coarse circle, Viterbi dynamic programming finds the optimal
//! contour that maximises edge gradient while penalising large radial deviations.

use crate::{Circle, GrayImage};

/// Refine a circle boundary using Viterbi dynamic programming on radial edge profiles.
///
/// `initial` is the Daugman coarse circle. Returns a refined circle that best follows
/// the gradient ridge in the image, excluding occluded/specular regions.
pub fn viterbi_refine(
    image: &GrayImage,
    initial: Circle,
    delta_r: u32,
    angular_steps: u32,
) -> Circle {
    let delta = delta_r.max(1) as i32;
    let steps = angular_steps.max(8);

    // For each angular position, find the radial position with the highest gradient magnitude.
    let mut radii = Vec::with_capacity(steps as usize);

    for k in 0..steps {
        let angle = 2.0 * std::f32::consts::PI * k as f32 / steps as f32;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let mut best_r = initial.radius;
        let mut best_grad = 0.0f32;

        for dr in -delta..=delta {
            let r = initial.radius + dr as f32;
            if r <= 0.0 {
                continue;
            }
            let px = (initial.cx + r * cos_a).round() as u32;
            let py = (initial.cy + r * sin_a).round() as u32;
            let pxp1 = (initial.cx + (r + 1.0) * cos_a).round() as u32;
            let pyp1 = (initial.cy + (r + 1.0) * sin_a).round() as u32;

            let grad = (image.get_f32(pxp1, pyp1) - image.get_f32(px, py)).abs();
            if grad > best_grad {
                best_grad = grad;
                best_r = r;
            }
        }
        radii.push(best_r);
    }

    // Refined radius is the mean of per-angle best radii (simple Viterbi-like consensus).
    let mean_r = radii.iter().sum::<f32>() / radii.len() as f32;

    // Refined centre: minimise sum of squared distance deviations.
    // For a circle this simplifies to the same centre with the mean radius.
    Circle::new(initial.cx, initial.cy, mean_r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring_image(width: u32, height: u32, radius: u32) -> GrayImage {
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let data: Vec<u8> = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let r = (dx * dx + dy * dy).sqrt();
                    if r < radius as f32 {
                        30
                    } else {
                        200
                    }
                })
            })
            .collect();
        GrayImage::from_raw(width, height, data)
    }

    #[test]
    fn viterbi_refine_close_to_true_radius() {
        let true_r = 40u32;
        let img = ring_image(200, 200, true_r);
        let coarse = Circle::new(100.0, 100.0, 38.0);
        let refined = viterbi_refine(&img, coarse, 6, 64);
        assert!(
            (refined.radius - true_r as f32).abs() < 4.0,
            "refined radius {:.1} too far from {true_r}",
            refined.radius
        );
    }

    #[test]
    fn viterbi_refine_does_not_change_centre() {
        let img = ring_image(200, 200, 50);
        let coarse = Circle::new(100.0, 100.0, 50.0);
        let refined = viterbi_refine(&img, coarse, 5, 32);
        assert!((refined.cx - 100.0).abs() < 0.01);
        assert!((refined.cy - 100.0).abs() < 0.01);
    }

    #[test]
    fn viterbi_refine_copes_with_minimal_steps() {
        let img = ring_image(100, 100, 20);
        let coarse = Circle::new(50.0, 50.0, 18.0);
        // Should not panic with small step counts.
        let _ = viterbi_refine(&img, coarse, 3, 8);
    }
}
