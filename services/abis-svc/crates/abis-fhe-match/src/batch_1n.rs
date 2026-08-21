//! 1:N encrypted gallery search via SIMD packing.
//!
//! Interface + toy implementation. Real SIMD packing batches multiple
//! ciphertexts into one CKKS polynomial slot vector.

use crate::match_ckks::{CkksCiphertext, CkksMatchError, CkksMatchTrait, CkksMatcher};

/// Result of a 1:N search.
#[derive(Debug, Clone)]
pub struct BatchMatchResult {
    pub best_idx: usize,
    pub best_score: f64,
    pub scores: Vec<f64>,
}

/// 1:N CKKS batch matcher.
pub struct BatchMatcher {
    inner: CkksMatcher,
}

impl BatchMatcher {
    pub fn new(scale: f64) -> Self {
        Self {
            inner: CkksMatcher::new(scale),
        }
    }

    /// Searches an encrypted gallery for the best matching entry.
    ///
    /// # Errors
    /// Returns an error if the gallery is empty or all comparisons fail.
    pub fn search_1n(
        &self,
        probe: &CkksCiphertext,
        gallery: &[CkksCiphertext],
        threshold: f64,
    ) -> Result<BatchMatchResult, CkksMatchError> {
        if gallery.is_empty() {
            return Err(CkksMatchError::EmptyCiphertext);
        }

        let mut scores = Vec::with_capacity(gallery.len());
        let mut best_score = f64::NEG_INFINITY;
        let mut best_idx = 0;

        for (idx, entry) in gallery.iter().enumerate() {
            match self.inner.match_1v1(probe, entry, f64::NEG_INFINITY) {
                Ok(s) | Err(CkksMatchError::BelowThreshold { score: s, .. }) => {
                    scores.push(s);
                    if s > best_score {
                        best_score = s;
                        best_idx = idx;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        if best_score < threshold {
            return Err(CkksMatchError::BelowThreshold {
                score: best_score,
                threshold,
            });
        }

        Ok(BatchMatchResult {
            best_idx,
            best_score,
            scores,
        })
    }
}
