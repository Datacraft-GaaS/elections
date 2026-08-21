use crate::models::RegisterSubset;

/// Generate a RegisterSubset from a BVR snapshot scoped to a constituency.
///
/// In production this calls M02 BVR via internal API to extract the snapshot
/// subset for the given constituency_id and snapshot_id. The boundary polygon
/// is validated against M18 before extraction (FR-M50-017).
///
/// This implementation returns a stub; wire M02 and M18 adapters for field builds.
///
/// Extraction must complete within 4 hours of by-election activation (FR-M50-020).
pub fn generate_subset(
    by_election_id: String,
    constituency_id: String,
    snapshot_id: String,
) -> RegisterSubset {
    let id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // TODO(prod): call M02 BVR snapshot API with constituency boundary polygon from M18.
    // Validate: subset voter_count must match M04 snapshot count for the polygon (FR-M50-021).
    // If discrepancy > 0, raise anomaly case.
    RegisterSubset {
        id,
        by_election_id,
        snapshot_id,
        constituency_id,
        station_count: 0, // populated from M02 response
        voter_count: 0,   // populated from M02 response
        signed_blob_hash: None,
        generated_at: now,
        frozen_at: None,
    }
}

/// Freeze the register subset at nomination close.
/// Once frozen, the write path is closed via OPA policy (FR-M50-018).
/// Returns the frozen subset with frozen_at timestamp set.
pub fn freeze_subset(mut subset: RegisterSubset) -> RegisterSubset {
    assert!(
        subset.frozen_at.is_none(),
        "subset {} is already frozen",
        subset.id
    );
    subset.frozen_at = Some(chrono::Utc::now().to_rfc3339());
    subset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_subset_returns_valid_stub() {
        let subset = generate_subset("be-1".into(), "const-1".into(), "snap-1".into());
        assert!(!subset.id.is_empty());
        assert_eq!(subset.by_election_id, "be-1");
        assert_eq!(subset.constituency_id, "const-1");
        assert!(subset.frozen_at.is_none());
    }

    #[test]
    fn freeze_sets_timestamp() {
        let subset = generate_subset("be-2".into(), "const-2".into(), "snap-2".into());
        let frozen = freeze_subset(subset);
        assert!(frozen.frozen_at.is_some());
    }

    #[test]
    #[should_panic(expected = "already frozen")]
    fn double_freeze_panics() {
        let subset = generate_subset("be-3".into(), "const-3".into(), "snap-3".into());
        let frozen = freeze_subset(subset);
        let _ = freeze_subset(frozen); // should panic
    }
}
