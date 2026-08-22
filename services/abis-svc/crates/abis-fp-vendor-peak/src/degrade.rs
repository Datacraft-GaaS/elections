//! Graceful degradation when one vendor is unhealthy.
//!
//! If exactly one vendor is unreachable, the remaining two still form a
//! valid 2-of-2 vote. The result is tagged with a reduced-confidence badge
//! so downstream consumers can apply a higher decision threshold or flag for
//! adjudication.

use serde::{Deserialize, Serialize};

/// Outcome of a degraded (< 3 vendors) consensus attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DegradeOutcome {
    /// Two vendors agreed; result is valid but at reduced confidence.
    TwoOfTwoAgreed,
    /// Only one vendor available; cannot form consensus.
    SingleVendorOnly,
    /// No vendors available.
    NoVendors,
}

/// Match candidate decorated with a degraded-confidence badge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradedCandidate {
    /// The calibrated consensus score from the available vendors.
    pub score: f64,
    /// Whether the consensus threshold was met despite degradation.
    pub accepted: bool,
    /// How many vendors voted.
    pub vendors_used: usize,
    /// Degradation classification.
    pub outcome: DegradeOutcome,
    /// Names of vendors that were unavailable.
    pub degraded_vendors: Vec<String>,
}

impl DegradedCandidate {
    /// Construct from a 2-of-2 result (one vendor down).
    pub fn two_of_two(score: f64, accepted: bool, degraded_vendor: String) -> Self {
        Self {
            score,
            accepted,
            vendors_used: 2,
            outcome: DegradeOutcome::TwoOfTwoAgreed,
            degraded_vendors: vec![degraded_vendor],
        }
    }

    /// Construct for single-vendor fallback (two vendors down).
    pub fn single_vendor(score: f64, degraded_vendors: Vec<String>) -> Self {
        Self {
            score,
            accepted: false,
            vendors_used: 1,
            outcome: DegradeOutcome::SingleVendorOnly,
            degraded_vendors,
        }
    }

    /// Returns true when the result carries full-confidence.
    pub fn is_full_confidence(&self) -> bool {
        self.vendors_used == 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_of_two_not_full_confidence() {
        let c = DegradedCandidate::two_of_two(0.9, true, "vendor_b".into());
        assert!(!c.is_full_confidence());
        assert_eq!(c.outcome, DegradeOutcome::TwoOfTwoAgreed);
        assert!(c.accepted);
    }

    #[test]
    fn single_vendor_always_rejected() {
        let c = DegradedCandidate::single_vendor(0.99, vec!["vendor_a".into(), "vendor_b".into()]);
        assert!(!c.accepted);
        assert_eq!(c.outcome, DegradeOutcome::SingleVendorOnly);
    }
}
