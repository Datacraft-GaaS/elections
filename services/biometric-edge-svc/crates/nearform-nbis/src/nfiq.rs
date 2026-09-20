//! NFIQ-style fingerprint quality screening.
//!
//! NOTE — estimator, not the C classifier: upstream does not ship the
//! `runmlp2()` implementation that `nfiq.c` needs (declared in
//! `nfiq/include/mlp.h`, defined nowhere in the mirror), so the C NFIQ core
//! is excluded from the build (see `build.rs`). This module maps extractor
//! output onto the NFIQ 1–5 scale (1 = excellent, 5 = poor) with documented,
//! tested thresholds. Bind the C classifier if `runmlp2` ever becomes
//! available; the mapping below is deliberately conservative (rejects more).

use super::template::{Minutia, Template};

/// NFIQ quality verdict (1 = excellent … 5 = poor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NfiqResult {
    /// NFIQ class 1..=5.
    pub score: u8,
    /// Mean minutiae quality 0..=100 backing the score.
    pub mean_quality: u32,
}

/// Assess a decoded template (estimator — see module docs).
///
/// Mapping (conservative by design):
/// - < 5 minutiae → 5 (mirrors NFIQ `MIN_MINUTIAE` / `TOO_FEW_MINUTIAE`)
/// - mean quality ≥ 80 with ≥ 25 minutiae → 1
/// - mean quality ≥ 60 with ≥ 12 minutiae → 2
/// - mean quality ≥ 40 → 3
/// - otherwise → 4
pub fn assess_quality(template: &Template) -> NfiqResult {
    assess_minutiae(&template.minutiae)
}

/// Estimator core over raw minutiae (testable without images).
pub fn assess_minutiae(minutiae: &[Minutia]) -> NfiqResult {
    if minutiae.len() < 5 {
        return NfiqResult { score: 5, mean_quality: 0 };
    }
    let mean =
        minutiae.iter().map(|m| m.quality as u64).sum::<u64>() / minutiae.len() as u64;
    let mean = mean as u32;
    let score = if mean >= 80 && minutiae.len() >= 25 {
        1
    } else if mean >= 60 && minutiae.len() >= 12 {
        2
    } else if mean >= 40 {
        3
    } else {
        4
    };
    NfiqResult { score, mean_quality: mean }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::MinutiaType;

    fn minutia(quality: u32) -> Minutia {
        Minutia {
            x: 10,
            y: 10,
            theta_deg: 90.0,
            quality,
            mclass: MinutiaType::RidgeEnd,
        }
    }

    #[test]
    fn too_few_minutiae_is_poor() {
        let ms: Vec<Minutia> = (0..4).map(|_| minutia(95)).collect();
        assert_eq!(assess_minutiae(&ms).score, 5);
    }

    #[test]
    fn excellent_needs_count_and_quality() {
        let ms: Vec<Minutia> = (0..30).map(|_| minutia(90)).collect();
        assert_eq!(assess_minutiae(&ms).score, 1);
        // High quality but too few minutiae cannot be excellent.
        let ms: Vec<Minutia> = (0..8).map(|_| minutia(95)).collect();
        assert!(assess_minutiae(&ms).score >= 2);
    }

    #[test]
    fn scale_is_monotone_in_quality() {
        let s = |q: u32| assess_minutiae(&(0..20).map(|_| minutia(q)).collect::<Vec<_>>()).score;
        assert!(s(90) <= s(65));
        assert!(s(65) <= s(45));
        assert!(s(45) <= s(10));
    }
}
