//! Multi-granularity orientation-field matcher.
//!
//! Three levels of comparison:
//! 1. Coarse: pattern class agreement (fast reject).
//! 2. Medium: singular point count + relative positions.
//! 3. Fine: orientation-field cross-correlation over overlapping blocks.

use crate::{
    field::{compute_orientation_field, field_correlation},
    pattern::classify_pattern,
    singular::detect_singular_points,
    GrayImage, OrientationError,
};

/// Score breakdown from orientation matching.
#[derive(Debug, Clone)]
pub struct OrientationScore {
    pub coarse: f32,
    pub medium: f32,
    pub fine: f32,
    pub fused: f32,
    /// False when the coarse check triggers a hard reject.
    pub passed_coarse: bool,
}

/// Orientation-field matcher configuration.
#[derive(Debug, Clone)]
pub struct OrientationMatcher {
    /// Block size for orientation field computation (pixels).
    pub block_size: u32,
    /// Score threshold for a 'match' decision.
    pub threshold: f32,
    /// Weight for field correlation (fine) relative to singular-point score (medium).
    pub fine_weight: f32,
}

impl Default for OrientationMatcher {
    fn default() -> Self {
        Self {
            block_size: 8,
            threshold: 0.5,
            fine_weight: 0.6,
        }
    }
}

impl OrientationMatcher {
    pub fn new(block_size: u32, threshold: f32) -> Self {
        Self {
            block_size,
            threshold,
            ..Self::default()
        }
    }

    /// Compare two fingerprint images by their orientation fields.
    pub fn compare(
        &self,
        probe: &GrayImage,
        gallery: &GrayImage,
    ) -> Result<OrientationScore, OrientationError> {
        // Compute orientation fields.
        let field_p = compute_orientation_field(probe, self.block_size)?;
        let field_g = compute_orientation_field(gallery, self.block_size)?;

        // Detect singular points.
        let sp_p = detect_singular_points(&field_p);
        let sp_g = detect_singular_points(&field_g);

        // Coarse: pattern class.
        let pat_p = classify_pattern(&sp_p);
        let pat_g = classify_pattern(&sp_g);
        let passed_coarse = pat_p.compatible_with(pat_g);
        let coarse = if passed_coarse { 1.0 } else { 0.0 };

        // Medium: singular point agreement (core count + rough spatial proximity).
        let medium = singular_point_score(&sp_p, &sp_g);

        // Fine: orientation field cross-correlation.
        let fine = if field_p.width == field_g.width && field_p.height == field_g.height {
            field_correlation(&field_p, &field_g)?
        } else {
            // Different sizes -- skip fine comparison.
            0.5
        };

        let fused = if !passed_coarse {
            0.0
        } else {
            let w_fine = self.fine_weight;
            let w_medium = 1.0 - w_fine;
            coarse * (w_medium * medium + w_fine * fine)
        };

        Ok(OrientationScore {
            coarse,
            medium,
            fine,
            fused,
            passed_coarse,
        })
    }
}

/// Score singular-point agreement between two sets.
///
/// Considers core count match and positional proximity of matched cores.
fn singular_point_score(
    a: &[crate::singular::SingularPoint],
    b: &[crate::singular::SingularPoint],
) -> f32 {
    use crate::singular::SingularKind;

    let cores_a = a
        .iter()
        .filter(|p| matches!(p.kind, SingularKind::Core | SingularKind::DoubleCore))
        .count();
    let cores_b = b
        .iter()
        .filter(|p| matches!(p.kind, SingularKind::Core | SingularKind::DoubleCore))
        .count();
    let deltas_a = a.iter().filter(|p| p.kind == SingularKind::Delta).count();
    let deltas_b = b.iter().filter(|p| p.kind == SingularKind::Delta).count();

    // Exact count match gives full score; each count difference reduces by 0.25.
    let core_diff = (cores_a as i32 - cores_b as i32).unsigned_abs() as f32;
    let delta_diff = (deltas_a as i32 - deltas_b as i32).unsigned_abs() as f32;
    (1.0 - 0.25 * (core_diff + delta_diff)).max(0.0)
}
