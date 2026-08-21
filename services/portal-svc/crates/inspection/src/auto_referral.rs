use chrono::{DateTime, Utc};
use std::collections::HashSet;
use uuid::Uuid;

use crate::domain::{AutoReferralPacket, CorrectionClaim, CorrectionStatus};

/// Idempotent auto-referral scheduler.
///
/// Scans pending sustained corrections and refers them to eCourt when
/// the polling day is within 60 days. Keyed by claim_id — double-firing
/// is safe because already-referred claims are skipped.
pub struct AutoReferralScheduler {
    /// In-memory idempotency set (production: backed by DB UNIQUE constraint).
    referred: HashSet<Uuid>,
    ecourt_base_url: String,
}

impl AutoReferralScheduler {
    pub fn new(ecourt_base_url: impl Into<String>) -> Self {
        Self {
            referred: HashSet::new(),
            ecourt_base_url: ecourt_base_url.into(),
        }
    }

    /// Returns true if a claim should be auto-referred.
    ///
    /// A claim qualifies when:
    ///  - status is `Sustained`
    ///  - it has not already been referred
    ///  - polling_day is within 60 days
    pub fn should_refer(&self, claim: &CorrectionClaim, polling_day: DateTime<Utc>) -> bool {
        if claim.status != CorrectionStatus::Approved {
            return false;
        }
        if self.referred.contains(&claim.id) {
            return false;
        }
        let days_until = (polling_day - Utc::now()).num_days();
        days_until <= 60 && days_until >= 0
    }

    /// Idempotently refer a claim. Returns None if already referred.
    pub fn refer(&mut self, claim: &CorrectionClaim) -> Option<AutoReferralPacket> {
        if self.referred.contains(&claim.id) {
            tracing::debug!(claim_id = %claim.id, "already referred — skipping");
            return None;
        }
        self.referred.insert(claim.id);
        let ecourt_ref = format!("{}/referrals/{}", self.ecourt_base_url, claim.id);
        tracing::info!(claim_id = %claim.id, %ecourt_ref, "auto-referred to eCourt");
        Some(AutoReferralPacket {
            id: Uuid::now_v7(),
            correction_claim_id: claim.id,
            ecourt_ref,
            referred_at: Utc::now(),
        })
    }

    /// Run the scheduler over a batch of claims.
    ///
    /// Returns the list of new referral packets created (skips already-referred).
    pub fn run_batch(
        &mut self,
        claims: &[CorrectionClaim],
        polling_day: DateTime<Utc>,
    ) -> Vec<AutoReferralPacket> {
        let eligible: Vec<&CorrectionClaim> = claims
            .iter()
            .filter(|c| self.should_refer(c, polling_day))
            .collect();
        eligible.into_iter().filter_map(|c| self.refer(c)).collect()
    }
}
