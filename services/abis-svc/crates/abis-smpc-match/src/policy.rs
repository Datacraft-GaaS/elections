//! SMPC policy: minimum threshold, party diversity, adversary bound.

use crate::session::SmpcSession;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmpcPolicyError {
    #[error("session is required")]
    MissingSession,
}

#[derive(Debug, Clone)]
pub struct SmpcPolicyOutcome {
    pub approved: bool,
    pub violations: Vec<String>,
}

/// Enforces SMPC session policy constraints.
pub struct SmpcPolicy {
    min_threshold: usize,
    min_party_count: usize,
    /// Maximum fraction of parties that can be adversarial (Byzantine bound).
    max_adversary_fraction: f64,
}

impl SmpcPolicy {
    pub fn new(min_threshold: usize, min_party_count: usize, max_adversary_fraction: f64) -> Self {
        Self {
            min_threshold,
            min_party_count,
            max_adversary_fraction,
        }
    }

    /// Default: 3-of-5, honest majority (at most 49% adversarial).
    /// SPDZ requires threshold > party_count/2; 3-of-5 satisfies that.
    pub fn default_3_of_5() -> Self {
        Self::new(3, 5, 0.49)
    }

    /// Checks a session against the policy.
    ///
    /// # Errors
    /// Currently infallible; returns `Result` for API consistency.
    pub fn check(&self, session: &SmpcSession) -> Result<SmpcPolicyOutcome, SmpcPolicyError> {
        let mut violations = Vec::new();

        if session.threshold < self.min_threshold {
            violations.push(format!(
                "threshold {} < minimum {}",
                session.threshold, self.min_threshold
            ));
        }
        if session.party_count < self.min_party_count {
            violations.push(format!(
                "party_count {} < minimum {}",
                session.party_count, self.min_party_count
            ));
        }

        let adversary_bound =
            (session.party_count as f64 * self.max_adversary_fraction).floor() as usize;
        let tolerated = (session.party_count - session.threshold).min(adversary_bound);
        if session.party_count - session.threshold > adversary_bound {
            violations.push(format!(
                "honest majority violated: {} non-threshold parties > adversary bound {}",
                session.party_count - session.threshold,
                tolerated
            ));
        }

        Ok(SmpcPolicyOutcome {
            approved: violations.is_empty(),
            violations,
        })
    }
}
