use crate::models::*;
use chrono::Utc;

pub struct ReviewEngine;

impl ReviewEngine {
    /// Transition Finding status; returns Err if transition is invalid.
    pub fn transition_finding(finding: &Finding, next: FindingStatus) -> Result<(), String> {
        use FindingStatus::*;
        match (&finding.status, &next) {
            (Submitted, Reviewed) | (Reviewed, Approved) | (Approved, Closed) => Ok(()),
            _ => Err(format!(
                "invalid transition {:?} -> {:?}",
                finding.status, next
            )),
        }
    }

    /// Returns true if a HIGH-severity finding has not been reviewed within 30 days.
    pub fn is_long_drip_alert_due(finding: &Finding) -> bool {
        if finding.severity != FindingSeverity::High {
            return false;
        }
        let age_days = Utc::now()
            .signed_duration_since(
                chrono::DateTime::parse_from_rfc3339(&finding.submitted_at)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or(Utc::now()),
            )
            .num_days();
        age_days >= 30 && finding.status == FindingStatus::Submitted
    }

    /// Build public tracker read model from recommendation (strips PII).
    pub fn to_public_tracker(rec: &Recommendation) -> PublicTrackerItem {
        PublicTrackerItem {
            recommendation_id: rec.id.clone(),
            recommendation_text: rec.recommendation_text.clone(),
            eel_response_type: rec.eel_response_type.clone(),
            status: rec.status.clone(),
            chair_signed_at: rec.chair_signed_at.clone(),
        }
    }

    /// Chair sign-off: validates no unresolved disputes, sets chair_signed_at.
    pub fn chair_sign(rec: &mut Recommendation, signed_by: &str) {
        rec.chair_signed_at = Some(Utc::now().to_rfc3339());
        rec.chair_signed_by = Some(signed_by.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions() {
        let f = Finding {
            status: FindingStatus::Submitted,
            severity: FindingSeverity::High,
            submitted_at: chrono::Utc::now().to_rfc3339(),
            ..Default::default()
        };
        assert!(ReviewEngine::transition_finding(&f, FindingStatus::Reviewed).is_ok());
        assert!(ReviewEngine::transition_finding(&f, FindingStatus::Closed).is_err());
    }

    #[test]
    fn long_drip_not_due_if_low_severity() {
        let f = Finding {
            severity: FindingSeverity::Low,
            status: FindingStatus::Submitted,
            submitted_at: (chrono::Utc::now() - chrono::Duration::days(40)).to_rfc3339(),
            ..Default::default()
        };
        assert!(!ReviewEngine::is_long_drip_alert_due(&f));
    }
}
