//! Bozorth3-style palm minutiae matcher adapted for palm biometrics.

use serde::{Deserialize, Serialize};

use crate::domain::{Minutia, PalmTemplate};

/// Rotation tolerance in degrees for minutiae pairing.
const ROT_TOL_DEG: f32 = 30.0;
/// Translation tolerance in pixels for minutiae pairing.
const TRANS_TOL_PX: f32 = 80.0;
/// Crease overlap threshold for applying bonus.
const CREASE_OVERLAP_THRESHOLD: f32 = 0.50;
/// Bonus added when crease patterns overlap sufficiently.
const CREASE_BONUS: f32 = 0.05;

/// Match score for a palm comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub raw: f32,
    pub threshold: f32,
    pub threshold_pass: bool,
}

/// Default score threshold for palm acceptance.
pub const DEFAULT_THRESHOLD: f32 = 0.40;

/// Compare two palm templates using a Bozorth3-style algorithm.
///
/// Returns a [Score] with raw value in [0.0, 1.0].
pub fn match_palm(probe: &PalmTemplate, gallery: &PalmTemplate) -> Score {
    let raw = compute_raw_score(
        &probe.minutiae,
        &gallery.minutiae,
        &probe.crease_pattern,
        &gallery.crease_pattern,
    );
    Score {
        raw,
        threshold: DEFAULT_THRESHOLD,
        threshold_pass: raw >= DEFAULT_THRESHOLD,
    }
}

fn angle_diff(a: u8, b: u8) -> f32 {
    let diff = (a as i16 - b as i16).abs();
    // angles are 0-179; wrap at 180
    let wrapped = if diff > 90 { 180 - diff } else { diff };
    wrapped as f32
}

fn compute_raw_score(
    probe: &[Minutia],
    gallery: &[Minutia],
    probe_crease: &[u8],
    gallery_crease: &[u8],
) -> f32 {
    if probe.is_empty() || gallery.is_empty() {
        return 0.0;
    }

    let mut matched = 0usize;
    let mut gallery_used = vec![false; gallery.len()];

    for pm in probe {
        for (gi, gm) in gallery.iter().enumerate() {
            if gallery_used[gi] {
                continue;
            }
            let dx = (pm.x as f32 - gm.x as f32).abs();
            let dy = (pm.y as f32 - gm.y as f32).abs();
            let dist = (dx * dx + dy * dy).sqrt();
            let rot = angle_diff(pm.angle, gm.angle);
            if dist <= TRANS_TOL_PX && rot <= ROT_TOL_DEG {
                matched += 1;
                gallery_used[gi] = true;
                break;
            }
        }
    }

    let denom = probe.len().min(gallery.len()) as f32;
    let base = matched as f32 / denom;

    let crease_bonus =
        if crease_overlap_fraction(probe_crease, gallery_crease) > CREASE_OVERLAP_THRESHOLD {
            CREASE_BONUS
        } else {
            0.0
        };

    (base + crease_bonus).min(1.0)
}

/// Compute the fraction of probe crease bytes that appear in gallery crease bytes.
fn crease_overlap_fraction(probe: &[u8], gallery: &[u8]) -> f32 {
    if probe.is_empty() || gallery.is_empty() {
        return 0.0;
    }
    let matching = probe
        .iter()
        .zip(gallery.iter())
        .filter(|(a, b)| a == b)
        .count();
    let denom = probe.len().min(gallery.len()) as f32;
    matching as f32 / denom
}
