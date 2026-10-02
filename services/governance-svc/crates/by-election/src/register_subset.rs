// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::models::RegisterSubset;

/// Initialise a RegisterSubset record for a by-election, scoped to a
/// constituency and pinned to an M02 BVR snapshot.
///
/// The record starts with station_count and voter_count at zero and no
/// integrity hash: those fields are populated by the extraction step, which
/// calls the M02 BVR snapshot API using the constituency polygon from M18
/// (FR-M50-016, FR-M50-017) and reconciles counts against the M04 snapshot
/// (FR-M50-021). No M02 or M18 adapter crate exists in this workspace yet,
/// so extraction lives with the integration layer that owns those clients.
///
/// Extraction must complete within 4 hours of by-election activation (FR-M50-020).
pub fn generate_subset(
    by_election_id: String,
    constituency_id: String,
    snapshot_id: String,
) -> RegisterSubset {
    let id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Extraction calls the M02 BVR snapshot API scoped by the M18 constituency
    // polygon and reconciles counts against M04 (FR-M50-016, FR-M50-017,
    // FR-M50-021). That step lives with the integration layer that owns the
    // M02/M18 clients; no adapter crate exists in this workspace.
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

/// Freeze the register subset at nomination close (FR-M50-018).
/// Once frozen, the write path is closed via OPA policy. Panics if the subset
/// is already frozen; the HTTP layer maps this to 403 freeze-violation.
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
