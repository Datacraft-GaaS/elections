//! Latent print quality scoring.

use crate::domain::LatentTemplate;

/// Compute a quality score in [0.0, 100.0] for a latent template.
///
/// Score = usable_ridge_pct * (1.0 - distortion_score) * minutiae_density,
/// where minutiae_density = clamp(count / 20.0, 0, 1).
pub fn score_quality(template: &LatentTemplate) -> f32 {
    if template.minutiae.is_empty() {
        return 0.0;
    }

    let minutiae_density = (template.minutiae.len() as f32 / 20.0).min(1.0);
    let clarity = (1.0 - template.distortion_score).max(0.0);
    (template.usable_ridge_pct * clarity * minutiae_density * 100.0).min(100.0)
}
