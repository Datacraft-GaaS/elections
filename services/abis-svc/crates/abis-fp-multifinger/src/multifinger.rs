//! Multi-finger template and per-finger scoring.
//!
//! Reference: ISO/IEC 19794-2:2011 multi-finger capture format.

use crate::slap::{BoundingBox, FingerPosition};

/// Per-finger match score with decision.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerFingerScore {
    pub position: FingerPosition,
    pub score: f32,
    pub is_match: bool,
}

/// A multi-finger template holding per-finger data.
///
/// Weights per position follow empirical NIST NFIQ2 sensitivity:
///   thumb 1.2, index 1.1, middle 1.0, ring 0.9, little 0.8
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiFingerTemplate {
    /// Per-finger entries in capture order.
    pub fingers: Vec<MultiFingerEntry>,
}

/// One finger's data within a multi-finger template.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiFingerEntry {
    pub position: FingerPosition,
    pub bounding_box: BoundingBox,
    /// Normalised quality score in [0.0, 1.0].
    pub quality: f32,
}

impl MultiFingerTemplate {
    pub fn new(fingers: Vec<MultiFingerEntry>) -> Self {
        Self { fingers }
    }

    pub fn finger_count(&self) -> usize {
        self.fingers.len()
    }
}

/// Position weight for fusion.  Higher weights on more discriminative fingers.
pub fn position_weight(pos: FingerPosition) -> f32 {
    match pos {
        FingerPosition::RightThumb | FingerPosition::LeftThumb => 1.2,
        FingerPosition::RightIndex | FingerPosition::LeftIndex => 1.1,
        FingerPosition::RightMiddle | FingerPosition::LeftMiddle => 1.0,
        FingerPosition::RightRing | FingerPosition::LeftRing => 0.9,
        FingerPosition::RightLittle | FingerPosition::LeftLittle => 0.8,
    }
}

/// Fuse per-finger scores into a single overall score using position-weighted average.
pub fn fuse_per_finger(scores: &[PerFingerScore]) -> f32 {
    if scores.is_empty() {
        return 0.0;
    }
    let (weighted_sum, weight_sum) = scores.iter().fold((0.0f32, 0.0f32), |(ws, wt), s| {
        let w = position_weight(s.position);
        (ws + w * s.score, wt + w)
    });
    if weight_sum < f32::EPSILON {
        0.0
    } else {
        weighted_sum / weight_sum
    }
}
