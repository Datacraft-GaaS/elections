//! TierGate: decides shielded vs plaintext review based on case severity.
//!
//! Tier-1 (election-critical) -> plaintext dual-review required.
//! Tier-2 -> shielded review acceptable.
//! Tier-3 -> shielded review default.

use serde::{Deserialize, Serialize};

/// Case severity tier, independent from ReviewerTier in abis-adjudication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CaseSeverity {
    /// Election-critical: requires plaintext dual-review.
    Tier1ElectionCritical,
    /// High-importance case: shielded review acceptable.
    Tier2High,
    /// Routine case: shielded review default.
    Tier3Routine,
}

/// The review mode determined by the TierGate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewMode {
    /// Both reviewers see plaintext templates. Used for Tier-1 only.
    PlaintextDualReview,
    /// Both reviewers see only Bloom-shielded comparisons.
    ShieldedReview,
}

/// Determine the review mode for a case based on severity.
///
/// This is the sole policy decision point: centralising it here ensures no
/// caller can accidentally choose plaintext for a low-severity case or
/// shielded for a critical one.
pub fn gate(severity: CaseSeverity) -> ReviewMode {
    match severity {
        CaseSeverity::Tier1ElectionCritical => ReviewMode::PlaintextDualReview,
        CaseSeverity::Tier2High | CaseSeverity::Tier3Routine => ReviewMode::ShieldedReview,
    }
}

/// Returns `true` if the review mode allows plaintext template access.
pub fn allows_plaintext(mode: ReviewMode) -> bool {
    mode == ReviewMode::PlaintextDualReview
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier1_requires_plaintext_dual_review() {
        assert_eq!(
            gate(CaseSeverity::Tier1ElectionCritical),
            ReviewMode::PlaintextDualReview
        );
    }

    #[test]
    fn tier2_allows_shielded() {
        assert_eq!(gate(CaseSeverity::Tier2High), ReviewMode::ShieldedReview);
    }

    #[test]
    fn tier3_defaults_to_shielded() {
        assert_eq!(gate(CaseSeverity::Tier3Routine), ReviewMode::ShieldedReview);
    }

    #[test]
    fn allows_plaintext_only_for_plaintext_mode() {
        assert!(allows_plaintext(ReviewMode::PlaintextDualReview));
        assert!(!allows_plaintext(ReviewMode::ShieldedReview));
    }

    #[test]
    fn tier1_is_most_severe() {
        assert!(CaseSeverity::Tier1ElectionCritical < CaseSeverity::Tier2High);
        assert!(CaseSeverity::Tier2High < CaseSeverity::Tier3Routine);
    }
}
