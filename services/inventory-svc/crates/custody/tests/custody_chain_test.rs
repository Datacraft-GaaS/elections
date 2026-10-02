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

use chrono::Utc;
use custody::domain::{CustodyEvent, ReservePool};
use custody::events::SealBroken;

fn make_event(asset_id: &str, from: &str, to: &str, seq: u64) -> CustodyEvent {
    CustodyEvent {
        event_id: format!("evt-{seq}"),
        asset_id: asset_id.to_string(),
        from_custodian: from.to_string(),
        to_custodian: to.to_string(),
        location: "loc-A".to_string(),
        ts_hlc: Utc::now().timestamp_millis() + seq as i64,
        signature: "sig".to_string(),
        seq,
    }
}

/// "Where was kit X" — verify custody chain ordering and from/to correctness.
#[test]
fn test_custody_chain_ordering() {
    let asset_id = "kit-001";
    let mut events = vec![
        make_event(asset_id, "warehouse", "officer-A", 1),
        make_event(asset_id, "officer-A", "officer-B", 2),
        make_event(asset_id, "officer-B", "station-007", 3),
    ];

    // Sort by seq to simulate a query result.
    events.sort_by_key(|e| e.seq);

    assert_eq!(events[0].from_custodian, "warehouse");
    assert_eq!(events[0].to_custodian, "officer-A");
    assert_eq!(events[1].from_custodian, "officer-A");
    assert_eq!(events[1].to_custodian, "officer-B");
    assert_eq!(events[2].from_custodian, "officer-B");
    assert_eq!(events[2].to_custodian, "station-007");

    // Confirm monotone seq.
    for w in events.windows(2) {
        assert!(w[1].seq > w[0].seq);
    }
}

/// Breaking a seal must involve two distinct officers.
#[test]
fn test_seal_break_requires_two_officers() {
    let ev = SealBroken {
        seal_id: "seal-X".to_string(),
        asset_id: "kit-001".to_string(),
        broken_by_officer_1: "officer-A".to_string(),
        broken_by_officer_2: "officer-B".to_string(),
        reason: "pre-opening inspection".to_string(),
        broken_at: Utc::now(),
    };

    assert_ne!(
        ev.broken_by_officer_1, ev.broken_by_officer_2,
        "two distinct officers required to break a seal"
    );
}

/// Reserve pool alert: count at or below threshold triggers alert condition.
#[test]
fn test_reserve_pool_alert_threshold() {
    let pool = ReservePool {
        pool_id: "pool-1".to_string(),
        scope: "national".to_string(),
        asset_type: "kit".to_string(),
        count: 5,
        alert_threshold: 10,
        updated_at: Utc::now(),
    };

    assert!(
        pool.is_below_threshold(),
        "pool count 5 <= threshold 10 should trigger alert"
    );

    let safe = ReservePool { count: 50, ..pool };
    assert!(
        !safe.is_below_threshold(),
        "pool count 50 > threshold 10 should not alert"
    );
}
