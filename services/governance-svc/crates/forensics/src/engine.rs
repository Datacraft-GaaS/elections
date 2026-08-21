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

    /// Trigger analysis run for a cycle (stub; real impl dispatches to py/forensics-ml).
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
