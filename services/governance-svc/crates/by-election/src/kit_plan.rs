use crate::models::{KitAssignment, KitPlan};

/// Generate a kit deployment plan for a by-election.
///
/// In production: queries M20 inventory for available kits and creates
/// per-station assignments. Minimum kit count = polling station count
/// for the constituency. Kit reservation in M20 ReservePool blocks those
/// kits from other by-elections (FR-M50-024).
///
/// This implementation returns a stub; wire M20 adapter for field builds.
///
/// Plan must be generated within 30 minutes of by-election creation (FR-M50-023).
pub fn generate_kit_plan(by_election_id: String, station_codes: Vec<String>) -> KitPlan {
    let plan_id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // TODO(prod): query M20 inventory for available kits; reserve in ReservePool.
    // Per-station acknowledgement tracked; unacknowledged assignments listed in ECC.
    let assignments: Vec<KitAssignment> = station_codes
        .into_iter()
        .map(|station_code| KitAssignment {
            kit_id: uuid::Uuid::now_v7().to_string(),
            station_code,
            station_gps_lat: 0.0,
            station_gps_lng: 0.0,
            expected_delivery_date: now.clone(),
            acknowledged: false,
        })
        .collect();

    let kit_ids: Vec<String> = assignments.iter().map(|a| a.kit_id.clone()).collect();

    KitPlan {
        id: plan_id,
        by_election_id,
        kit_ids,
        station_assignments: assignments,
        signed_blob_hash: None,
        signed_at: None,
    }
}

/// Sign the kit plan via HSM (P06). In production uses eel-crypto-rs HSM adapter.
/// Returns the plan with signed_blob_hash populated.
/// HSM unavailable: signing queued in degraded mode per FR-M50-053.
pub fn sign_kit_plan(mut plan: KitPlan) -> KitPlan {
    // TODO(prod): sign canonical JSON of plan with P06 HSM; populate signed_blob_hash.
    let mock_hash = format!("sha256:stub-{}", plan.id);
    plan.signed_blob_hash = Some(mock_hash);
    plan.signed_at = Some(chrono::Utc::now().to_rfc3339());
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_one_kit_per_station() {
        let stations: Vec<String> = (1..=5).map(|i| format!("STN-{i:03}")).collect();
        let plan = generate_kit_plan("be-1".into(), stations.clone());
        assert_eq!(plan.station_assignments.len(), 5);
        assert_eq!(plan.kit_ids.len(), 5);
        assert_eq!(plan.by_election_id, "be-1");
        // All start unacknowledged
        assert_eq!(plan.unacknowledged_count(), 5);
    }

    #[test]
    fn sign_populates_hash() {
        let plan = generate_kit_plan("be-2".into(), vec!["STN-001".into()]);
        let signed = sign_kit_plan(plan);
        assert!(signed.signed_blob_hash.is_some());
        assert!(signed.signed_at.is_some());
    }

    #[test]
    fn empty_station_list_gives_empty_plan() {
        let plan = generate_kit_plan("be-3".into(), vec![]);
        assert_eq!(plan.station_assignments.len(), 0);
        assert_eq!(plan.unacknowledged_count(), 0);
    }
}
