//! Minimum agreement policy for consensus matching.
//!
//! Requires at least K of N sub-matchers to vote 'match' regardless of the
//! fused score.  This guards against a single outlier matcher inflating the
//! fused score past the decision threshold.

/// Policy: require at least `min_votes` matchers to vote 'match'.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MinAgreementPolicy {
    /// Minimum number of matchers that must vote 'match'.
    pub min_votes: usize,
}

impl Default for MinAgreementPolicy {
    fn default() -> Self {
        // K=3 of N=5 by default (strict majority).
        Self { min_votes: 3 }
    }
}

impl MinAgreementPolicy {
    pub fn new(min_votes: usize) -> Self {
        Self { min_votes }
    }

    /// Return true when the agreement threshold is met.
    ///
    /// The check is scaled: if fewer than 5 matchers ran, the minimum is
    /// proportionally reduced (floor) so partial inputs still work.
    pub fn check(&self, votes: usize, total: usize) -> bool {
        if total == 0 {
            return false;
        }
        // Scale min_votes proportionally to available matchers, rounding up.
        // No cap at total: if min_votes is large enough that the scaled value
        // exceeds total, the policy fails (not enough matchers ran).
        let effective_min = ((self.min_votes as f32 / 5.0) * total as f32).ceil() as usize;
        votes >= effective_min
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_matchers_requires_three() {
        let p = MinAgreementPolicy::new(3);
        assert!(!p.check(2, 5));
        assert!(p.check(3, 5));
        assert!(p.check(5, 5));
    }

    #[test]
    fn two_matchers_scales_min() {
        let p = MinAgreementPolicy::new(3);
        // 3/5 * 2 = 1.2 -> ceil = 2
        assert!(!p.check(1, 2));
        assert!(p.check(2, 2));
    }

    #[test]
    fn zero_total_returns_false() {
        let p = MinAgreementPolicy::default();
        assert!(!p.check(0, 0));
    }
}
