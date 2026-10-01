use crate::models::*;

pub struct ForensicsEngine;

impl ForensicsEngine {
    /// Build ODPP referral packet metadata (stub; real impl generates PDF bundle).
    pub fn build_odpp_packet(cycle: &AnalysisCycle, basis: &str) -> ReferralPacket {
        ReferralPacket {
            id: format!("ref-odpp-{}", cycle.id),
            cycle_id: cycle.id.clone(),
            referred_to: ReferralTarget::Odpp,
            referral_basis: basis.to_string(),
            content_url: format!("odpp://referral/{}", cycle.id),
            referred_at: chrono::Utc::now().to_rfc3339(),
            acknowledged_at: None,
        }
    }

    /// Validate cycle is complete before publishing replication package.
    pub fn can_publish_replication(cycle: &AnalysisCycle) -> bool {
        cycle.status == CycleStatus::Complete
    }

    /// Trigger analysis run for a cycle.
    ///
    /// Transitions the cycle to `Running` and stamps `started_at`. The estimate
    /// itself is produced by `py/forensics-ml`, a separate Python distribution;
    /// see `AnalysisCycle::blocked_reason`.
    pub fn start_run(cycle: &mut AnalysisCycle) {
        cycle.status = CycleStatus::Running;
        cycle.started_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odpp_packet_generation() {
        let cycle = AnalysisCycle {
            id: "cyc-001".to_string(),
            method: AnalysisMethod::Rd,
            status: CycleStatus::Complete,
            ..Default::default()
        };
        let packet = ForensicsEngine::build_odpp_packet(&cycle, "statistical anomaly");
        assert_eq!(packet.referred_to, ReferralTarget::Odpp);
        assert!(!packet.referral_basis.is_empty());
    }

    #[test]
    fn wire_values_match_the_contract() {
        // Pinned because `SCREAMING_SNAKE_CASE` would turn `DiD` into `DI_D`,
        // which is an accident of the variant name rather than a chosen value.
        assert_eq!(serde_json::to_string(&AnalysisMethod::DiD).unwrap(), "\"DIFF_IN_DIFF\"");
        assert_eq!(serde_json::to_string(&AnalysisMethod::Rd).unwrap(), "\"RD\"");
        assert_eq!(serde_json::from_str::<AnalysisMethod>("\"DIFF_IN_DIFF\"").unwrap(), AnalysisMethod::DiD);
        assert_eq!(serde_json::from_str::<AnalysisMethod>("\"DI_D\"").unwrap_err().to_string().contains("DIFF_IN_DIFF"), true);
        // CycleStatus is the one enum in this file serialized snake_case.
        assert_eq!(serde_json::to_string(&CycleStatus::Running).unwrap(), "\"running\"");
        assert_eq!(AnalysisMethod::DiD.wire(), "DIFF_IN_DIFF");
        assert_eq!(CycleStatus::Draft.wire(), "draft");
    }

    #[test]
    fn replication_requires_complete_cycle() {
        let draft = AnalysisCycle {
            status: CycleStatus::Draft,
            ..Default::default()
        };
        let complete = AnalysisCycle {
            status: CycleStatus::Complete,
            ..Default::default()
        };
        assert!(!ForensicsEngine::can_publish_replication(&draft));
        assert!(ForensicsEngine::can_publish_replication(&complete));
    }
}
