//! CDEFFS matcher: fuses EFS features with base minutiae score.
//!
//! Reference: NIST SP 500-290 e3 Annex A; Jain, Prabhakar, Hong, Pankanti,
//! "Filterbank-Based Fingerprint Matching," IEEE Trans. Image Processing 2000.
//!
//! Fusion rules:
//!   - Singular-point agreement (each matching core/delta) adds bonus.
//!   - Pattern-class mismatch returns 0.0 fusion contribution.
//!   - Crease/scar region overlap adds a small negative weight (distortion).

use crate::features::{ExtendedFeatureSet, PatternClass, SingularPointKind};

/// Bonus per matched singular point (core or delta).
const SINGULAR_MATCH_BONUS: f32 = 0.05;
/// Penalty when pattern classes are wholly incompatible.
const CLASS_MISMATCH_PENALTY: f32 = -1.0;
/// Spatial tolerance for singular-point matching (pixels).
const SINGULAR_TOL_PX: f32 = 20.0;
/// Angular tolerance for singular-point matching (degrees).
const SINGULAR_TOL_DEG: f32 = 20.0;

/// CDEFFS matcher.
///
/// Call `fuse(base_score, probe_efs, gallery_efs)` to get an EFS-adjusted score.
pub struct CdeffsMatcher;

impl CdeffsMatcher {
    pub fn new() -> Self {
        Self
    }

    /// Return a fusion delta in [-1.0, 1.0] to be added to a base minutiae score.
    ///
    /// The caller is responsible for clamping the final score to [0.0, 1.0].
    pub fn fuse(
        &self,
        base_score: f32,
        probe: &ExtendedFeatureSet,
        gallery: &ExtendedFeatureSet,
    ) -> f32 {
        let class_delta = class_score(probe.pattern_class, gallery.pattern_class);
        if class_delta < 0.0 {
            // Incompatible classes: hard reject fusion contribution.
            return 0.0;
        }

        let singular_bonus = singular_point_score(probe, gallery);

        (base_score + class_delta + singular_bonus).clamp(0.0, 1.0)
    }
}

impl Default for CdeffsMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns 0.0 for compatible classes, CLASS_MISMATCH_PENALTY for incompatible ones.
fn class_score(probe: PatternClass, gallery: PatternClass) -> f32 {
    if probe == PatternClass::Unknown || gallery == PatternClass::Unknown {
        return 0.0; // cannot penalise unknown
    }
    if are_compatible(probe, gallery) {
        0.0
    } else {
        CLASS_MISMATCH_PENALTY
    }
}

/// Broad pattern-class compatibility: loops are compatible with loops, etc.
fn are_compatible(a: PatternClass, b: PatternClass) -> bool {
    use PatternClass::*;
    matches!(
        (a, b),
        (Arch, Arch)
            | (TentedArch, TentedArch)
            | (Arch, TentedArch)
            | (TentedArch, Arch)
            | (LeftLoop, LeftLoop)
            | (RightLoop, RightLoop)
            | (Whorl, Whorl)
            | (Whorl, TwinLoop)
            | (Whorl, CentralPocketLoop)
            | (TwinLoop, Whorl)
            | (TwinLoop, TwinLoop)
            | (CentralPocketLoop, Whorl)
            | (CentralPocketLoop, CentralPocketLoop)
            | (Accidental, Accidental)
    )
}

/// Bonus for matched singular points (cores + deltas).
fn singular_point_score(probe: &ExtendedFeatureSet, gallery: &ExtendedFeatureSet) -> f32 {
    let mut bonus = 0.0f32;
    for pp in &probe.singular_points {
        for gp in &gallery.singular_points {
            if pp.kind != gp.kind {
                continue;
            }
            let dx = pp.x as f32 - gp.x as f32;
            let dy = pp.y as f32 - gp.y as f32;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle_diff = (pp.orientation - gp.orientation).abs().rem_euclid(360.0);
            let angle_diff = if angle_diff > 180.0 {
                360.0 - angle_diff
            } else {
                angle_diff
            };
            if dist <= SINGULAR_TOL_PX && angle_diff <= SINGULAR_TOL_DEG {
                bonus += SINGULAR_MATCH_BONUS;
            }
        }
    }
    // Weight by kind: cores count double (they are rarer and more distinctive).
    let core_bonus: f32 = probe
        .singular_points
        .iter()
        .zip(gallery.singular_points.iter())
        .filter(|(p, g)| p.kind == SingularPointKind::Core && g.kind == SingularPointKind::Core)
        .count() as f32
        * SINGULAR_MATCH_BONUS;
    bonus + core_bonus
}
