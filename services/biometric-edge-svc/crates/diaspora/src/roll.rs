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

//! DiasporaRoll management: import, freeze, and snapshot hashing (WP-5.8, M24).

use crate::domain::DiasporaRoll;
use blake3::Hasher;
use chrono::{DateTime, Utc};
use uuid::Uuid;

impl DiasporaRoll {
    /// Create a new (unfrozen) roll for the given station.
    pub fn new(embassy_station_id: Uuid, voter_count: u64) -> Self {
        DiasporaRoll {
            id: Uuid::now_v7(),
            embassy_station_id,
            snapshot_taken_at: Utc::now(),
            voter_count,
            hash: String::new(),
            frozen: false,
        }
    }

    /// Freeze the roll: compute the snapshot hash and set frozen = true.
    /// Idempotent: a second call returns an unchanged clone.
    pub fn freeze(self, _now: DateTime<Utc>) -> DiasporaRoll {
        if self.frozen {
            return self;
        }
        // Hash is already set at import time via snapshot_from_voter_ids.
        // Freeze just locks the record.
        DiasporaRoll {
            frozen: true,
            ..self
        }
    }
}

/// Compute a BLAKE3 hash over a sorted list of voter IDs (strings).
/// Produces a deterministic 64-character hex digest.
pub fn snapshot_hash(voter_ids: &mut [String]) -> String {
    voter_ids.sort_unstable();
    let mut hasher = Hasher::new();
    for id in voter_ids.iter() {
        hasher.update(id.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// Build a DiasporaRoll from a list of voter IDs, computing the hash.
pub fn roll_from_voter_ids(embassy_station_id: Uuid, mut voter_ids: Vec<String>) -> DiasporaRoll {
    let hash = snapshot_hash(&mut voter_ids);
    DiasporaRoll {
        id: Uuid::now_v7(),
        embassy_station_id,
        snapshot_taken_at: Utc::now(),
        voter_count: voter_ids.len() as u64,
        hash,
        frozen: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn freeze_is_idempotent() {
        let roll = roll_from_voter_ids(Uuid::now_v7(), vec!["V001".into(), "V002".into()]);
        let frozen_once = roll.clone().freeze(Utc::now());
        assert!(frozen_once.frozen);
        let frozen_twice = frozen_once.clone().freeze(Utc::now());
        assert_eq!(frozen_once.hash, frozen_twice.hash);
        assert_eq!(frozen_once.voter_count, frozen_twice.voter_count);
        assert_eq!(frozen_once.id, frozen_twice.id);
        assert!(frozen_twice.frozen);
    }

    #[test]
    fn hash_is_order_independent() {
        let h1 = snapshot_hash(&mut ["B".into(), "A".into(), "C".into()]);
        let h2 = snapshot_hash(&mut ["A".into(), "C".into(), "B".into()]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_changes_with_different_voters() {
        let h1 = snapshot_hash(&mut ["V001".into()]);
        let h2 = snapshot_hash(&mut ["V002".into()]);
        assert_ne!(h1, h2);
    }
}
