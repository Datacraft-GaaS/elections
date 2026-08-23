use crate::models::{KitAssignment, KitPlan};

/// Generate a kit deployment plan for a by-election: one kit assignment per
/// polling station, all starting unacknowledged. Minimum kit count therefore
/// equals the polling station count supplied by the caller, which mirrors the
/// M20 inventory rule (FR-M50-023, FR-M50-024). The M20 ReservePool reservation
/// call is an integration concern owned by the HTTP layer; no M20 adapter crate
/// exists in this workspace yet, so reservation is not performed here.
///
/// Plan must be generated within 30 minutes of by-election creation (FR-M50-023).
pub fn generate_kit_plan(by_election_id: String, station_codes: Vec<String>) -> KitPlan {
    let plan_id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // M20 inventory query and ReservePool reservation are owned by the HTTP
    // layer, which holds the M20 client; no adapter crate exists in this
    // workspace. Per-station acknowledgement is tracked on each KitAssignment
    // and unacknowledged assignments surface in ECC outstanding-actions
    // (FR-M50-024, FR-M50-025).
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

/// Bind the kit plan to an integrity hash over its canonical JSON encoding
/// (FR-M50-025). In production the P06 HSM countersigns this hash via the
/// eel-crypto-rs HSM adapter; that adapter crate is not part of this workspace,
/// so this function records the SHA-256 content digest itself. Per FR-M50-053
/// the HSM countersign can be queued later without invalidating the digest:
/// any tampering with plan contents changes the hash and fails verification.
///
/// Returns the plan with signed_blob_hash and signed_at populated.
pub fn sign_kit_plan(mut plan: KitPlan) -> KitPlan {
    use sha2::{Digest, Sha256};

    let canonical =
        serde_json::to_vec(&plan).expect("KitPlan is JSON-serialisable by construction");
    let mut hasher = Sha256::new();
    hasher.update(&canonical);
    plan.signed_blob_hash = Some(format!("sha256:{}", hex::encode(hasher.finalize())));
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

#[cfg(test)]
mod sign_tests {
    use super::*;

    #[test]
    fn signed_hash_matches_canonical_json_digest() {
        use sha2::{Digest, Sha256};

        let plan = generate_kit_plan("be-9".into(), vec!["STN-001".into(), "STN-002".into()]);
        let expected = {
            let canonical = serde_json::to_vec(&plan).expect("serialisable");
            let mut h = Sha256::new();
            h.update(&canonical);
            format!("sha256:{}", hex::encode(h.finalize()))
        };
        let signed = sign_kit_plan(plan);
        assert_eq!(signed.signed_blob_hash.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn tampered_plan_changes_hash() {
        let plan = generate_kit_plan("be-10".into(), vec!["STN-001".into()]);
        let signed = sign_kit_plan(plan.clone());
        let mut tampered = plan;
        tampered.station_assignments[0].station_code = "STN-999".into();
        let resigned = sign_kit_plan(tampered);
        assert_ne!(signed.signed_blob_hash, resigned.signed_blob_hash);
    }
}
