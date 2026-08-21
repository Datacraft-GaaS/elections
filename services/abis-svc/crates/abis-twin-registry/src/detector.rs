//! Automated twin-candidate detection and face-iris correlation check.

use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::TwinCandidate;
use crate::error::TwinRegistryError;

/// Score range in which twins are ambiguous: above impostor zone, below match threshold.
pub const TWIN_AMBIGUITY_LOWER: f64 = 0.60;
pub const TWIN_AMBIGUITY_UPPER: f64 = 0.85;

/// Detect twin candidates from 1:N result pairs.
/// A candidate pair must score in the ambiguity band AND share demographic or geographic context.
pub fn detect_twin_candidates(
    pairs: &[(Uuid, Uuid, f64)],
    demographics_overlap: &HashMap<(Uuid, Uuid), bool>,
    geographic_overlap: &HashMap<(Uuid, Uuid), bool>,
) -> Vec<TwinCandidate> {
    pairs
        .iter()
        .filter(|(a, b, score)| {
            *score >= TWIN_AMBIGUITY_LOWER && *score <= TWIN_AMBIGUITY_UPPER && {
                let key = (*a, *b);
                let key_r = (*b, *a);
                let demo = demographics_overlap
                    .get(&key)
                    .or_else(|| demographics_overlap.get(&key_r))
                    .copied()
                    .unwrap_or(false);
                let geo = geographic_overlap
                    .get(&key)
                    .or_else(|| geographic_overlap.get(&key_r))
                    .copied()
                    .unwrap_or(false);
                demo || geo
            }
        })
        .map(|(a, b, score)| {
            let key = (*a, *b);
            let key_r = (*b, *a);
            TwinCandidate {
                subject_a: *a,
                subject_b: *b,
                match_score: *score,
                demographic_overlap: demographics_overlap
                    .get(&key)
                    .or_else(|| demographics_overlap.get(&key_r))
                    .copied()
                    .unwrap_or(false),
                geographic_overlap: geographic_overlap
                    .get(&key)
                    .or_else(|| geographic_overlap.get(&key_r))
                    .copied()
                    .unwrap_or(false),
                detected_at: Utc::now(),
            }
        })
        .collect()
}

/// Face-iris correlation check.
/// Genuine twins have high face similarity but non-trivial iris divergence (iris patterns differ).
/// Returns true if the pair passes the correlation check (likely twins, not impostor).
pub fn face_iris_correlation_check(
    face_score: f64,
    iris_score: f64,
    threshold: f64,
) -> Result<bool, TwinRegistryError> {
    if !(0.0..=1.0).contains(&face_score) || !(0.0..=1.0).contains(&iris_score) {
        return Err(TwinRegistryError::CorrelationCheckFailed(
            "scores must be in [0, 1]".to_string(),
        ));
    }
    let delta = (face_score - iris_score).abs();
    Ok(face_score > threshold && delta > 0.05)
}
