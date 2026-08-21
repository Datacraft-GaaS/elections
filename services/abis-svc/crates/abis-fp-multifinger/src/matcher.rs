//! Multi-finger matcher: compare per-finger images and fuse scores.

use crate::{
    dpr::{DprPolicy, DprResult},
    multifinger::{fuse_per_finger, position_weight, PerFingerScore},
    slap::{segment_slap, BoundingBox, FingerPosition, SlapKind},
    GrayImage, MultiFingerError,
};

/// Result of a multi-finger comparison.
#[derive(Debug, Clone)]
pub struct MultiFingerMatchResult {
    pub overall_score: f32,
    pub per_finger: Vec<PerFingerScore>,
    pub count_matched: usize,
    pub dpr: DprResult,
}

/// A simple per-finger scorer: computes normalised mean absolute difference of pixel histograms.
///
/// In production this would call a real minutiae or texture matcher per finger.
/// Here a lightweight histogram similarity serves as a structurally correct placeholder
/// that avoids pulling in heavy dependencies.
fn image_similarity(a: &GrayImage, b: &GrayImage) -> f32 {
    // Resize to same width for comparison by sampling.
    let len = 256;
    let mut ha = vec![0u32; len];
    let mut hb = vec![0u32; len];
    let step_a = (a.pixels.len() as f32 / 1000.0).max(1.0) as usize;
    let step_b = (b.pixels.len() as f32 / 1000.0).max(1.0) as usize;
    for &p in a.pixels.iter().step_by(step_a) {
        ha[p as usize] += 1;
    }
    for &p in b.pixels.iter().step_by(step_b) {
        hb[p as usize] += 1;
    }
    let sa: f32 = ha.iter().sum::<u32>() as f32;
    let sb: f32 = hb.iter().sum::<u32>() as f32;
    if sa == 0.0 || sb == 0.0 {
        return 0.0;
    }
    let corr: f32 = ha
        .iter()
        .zip(hb.iter())
        .map(|(&a, &b)| (a as f32 / sa) * (b as f32 / sb))
        .sum::<f32>();
    // Bhattacharyya coefficient -- 1 for identical histograms.
    corr.sqrt().min(1.0)
}

/// Multi-finger matcher configuration.
#[derive(Debug, Clone)]
pub struct MultiFingerMatcher {
    /// Per-finger score threshold to count a finger as 'matched'.
    pub per_finger_threshold: f32,
    /// DPR policy.
    pub dpr: DprPolicy,
}

impl Default for MultiFingerMatcher {
    fn default() -> Self {
        Self {
            per_finger_threshold: 0.4,
            dpr: DprPolicy::default(),
        }
    }
}

impl MultiFingerMatcher {
    pub fn new(per_finger_threshold: f32) -> Self {
        Self {
            per_finger_threshold,
            ..Self::default()
        }
    }

    /// Compare two slap images of the same kind.
    pub fn compare_slaps(
        &self,
        probe: &GrayImage,
        gallery: &GrayImage,
        kind: SlapKind,
    ) -> Result<MultiFingerMatchResult, MultiFingerError> {
        let probe_fingers = segment_slap(probe, kind)?;
        let gallery_fingers = segment_slap(gallery, kind)?;

        if probe_fingers.len() != gallery_fingers.len() {
            return Err(MultiFingerError::FingerCountMismatch {
                probe: probe_fingers.len(),
                gallery: gallery_fingers.len(),
            });
        }

        let mut per_finger: Vec<PerFingerScore> = Vec::with_capacity(probe_fingers.len());
        for ((pos, _bb_p, img_p), (_pos_g, _bb_g, img_g)) in
            probe_fingers.iter().zip(gallery_fingers.iter())
        {
            let score = image_similarity(img_p, img_g);
            let is_match = score >= self.per_finger_threshold;
            per_finger.push(PerFingerScore {
                position: *pos,
                score,
                is_match,
            });
        }

        let count_matched = per_finger.iter().filter(|s| s.is_match).count();
        let overall_score = fuse_per_finger(&per_finger);

        let available = probe_fingers.len();
        let dpr = self.dpr.evaluate(count_matched, available);

        Ok(MultiFingerMatchResult {
            overall_score,
            per_finger,
            count_matched,
            dpr,
        })
    }

    /// Compare two pre-segmented per-finger score lists (for testing and fusion).
    pub fn compare_per_finger_scores(
        &self,
        per_finger: Vec<PerFingerScore>,
        available: usize,
    ) -> MultiFingerMatchResult {
        let count_matched = per_finger.iter().filter(|s| s.is_match).count();
        let overall_score = fuse_per_finger(&per_finger);
        let dpr = self.dpr.evaluate(count_matched, available);
        MultiFingerMatchResult {
            overall_score,
            per_finger,
            count_matched,
            dpr,
        }
    }
}
