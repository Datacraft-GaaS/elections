//! RO clearance workflow with KRA and EACC integration doubles.
//! FR-M12-011, FR-M12-005, FR-M12-006, AC-M12-08.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClearanceSource {
    Kra,
    Eacc,
    Court,
    Ro,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClearanceVerdict {
    Cleared,
    Rejected,
    /// KRA/EACC unavailable; officer applied manual override with reason.
    ManualOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearanceDecision {
    pub id: Uuid,
    pub nomination_id: Uuid,
    pub source: ClearanceSource,
    pub verdict: ClearanceVerdict,
    pub evidence_ref: Option<String>,
    pub officer_id: String,
    pub override_reason: Option<String>,
    pub decided_at: DateTime<Utc>,
}

impl ClearanceDecision {
    pub fn new(
        nomination_id: Uuid,
        source: ClearanceSource,
        verdict: ClearanceVerdict,
        evidence_ref: Option<String>,
        officer_id: String,
        override_reason: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            nomination_id,
            source,
            verdict,
            evidence_ref,
            officer_id,
            override_reason,
            decided_at: Utc::now(),
        }
    }

    /// A manual override must always carry a reason (FR-M12-005, FR-M12-006).
    pub fn validate(&self) -> Result<(), String> {
        if self.verdict == ClearanceVerdict::ManualOverride && self.override_reason.is_none() {
            return Err("manual override requires a reason".into());
        }
        Ok(())
    }
}

/// KRA iTax adapter double — returns advisory clearance status.
/// Live adapter lives in integrations/kra-itax/.
pub async fn kra_advisory_check(id_number_hash: &str) -> Result<ClearanceVerdict, String> {
    // Double: always returns Cleared for integration tests.
    let _ = id_number_hash;
    Ok(ClearanceVerdict::Cleared)
}

/// EACC adapter double — async advisory; circuit-breaker handled by caller.
pub async fn eacc_advisory_check(id_number_hash: &str) -> Result<ClearanceVerdict, String> {
    let _ = id_number_hash;
    Ok(ClearanceVerdict::Cleared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    fn decision(verdict: ClearanceVerdict, reason: Option<String>) -> ClearanceDecision {
        ClearanceDecision::new(
            Uuid::new_v4(),
            ClearanceSource::Ro,
            verdict,
            None,
            "officer-1".into(),
            reason,
        )
    }

    #[trace("FR-M12-005")]
    #[test]
    fn manual_override_without_reason_is_rejected() {
        let d = decision(ClearanceVerdict::ManualOverride, None);
        assert!(d.validate().is_err());
    }

    #[trace("FR-M12-005")]
    #[test]
    fn manual_override_with_reason_is_accepted() {
        let d = decision(
            ClearanceVerdict::ManualOverride,
            Some("KRA unavailable; verified via gazette".into()),
        );
        assert!(d.validate().is_ok());
    }

    #[trace("FR-M12-006")]
    #[test]
    fn cleared_and_rejected_need_no_reason() {
        for verdict in [ClearanceVerdict::Cleared, ClearanceVerdict::Rejected] {
            let d = decision(verdict, None);
            assert!(d.validate().is_ok());
        }
    }
}
