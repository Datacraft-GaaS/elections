//! FMR/FNMR harness for ABIS accuracy evaluation.

use crate::domain::MatchOutcome;
use crate::pipeline::MatchPipeline;

pub struct FmrFnmrHarness {
    pub fmr: f64,
    pub fnmr: f64,
    pub total_genuine: u64,
    pub total_impostor: u64,
}

/// Compute FMR (false match rate) and FNMR (false non-match rate).
///
/// - genuine_pairs: pairs that should match (same person). FNMR = misses / total.
/// - impostor_pairs: pairs that should not match (different persons). FMR = false accepts / total.
pub fn compute_fmr_fnmr(
    matcher: &dyn MatchPipeline,
    genuine_pairs: &[(String, String)],
    impostor_pairs: &[(String, String)],
    threshold: f32,
) -> FmrFnmrHarness {
    let mut false_rejects: u64 = 0;
    for (probe, gallery) in genuine_pairs {
        let result = matcher.match_11(probe, gallery, threshold);
        if result.outcome != MatchOutcome::Match {
            false_rejects += 1;
        }
    }

    let mut false_accepts: u64 = 0;
    for (probe, gallery) in impostor_pairs {
        let result = matcher.match_11(probe, gallery, threshold);
        if result.outcome == MatchOutcome::Match {
            false_accepts += 1;
        }
    }

    let total_genuine = genuine_pairs.len() as u64;
    let total_impostor = impostor_pairs.len() as u64;

    FmrFnmrHarness {
        fmr: if total_impostor > 0 {
            false_accepts as f64 / total_impostor as f64
        } else {
            0.0
        },
        fnmr: if total_genuine > 0 {
            false_rejects as f64 / total_genuine as f64
        } else {
            0.0
        },
        total_genuine,
        total_impostor,
    }
}
