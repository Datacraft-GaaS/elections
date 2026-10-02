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
use eel_eventstore_rs::{Aggregate, StoredEvent};
use rts_domain::aggregates::form34a::{Form34A, Form34AStatus};
use rts_domain::events::EVENT_FORM34A_CAPTURED;
use serde_json::json;
use uuid::Uuid;

fn make_captured_event(stream_id: Uuid) -> StoredEvent {
    StoredEvent {
        stream_id, stream_version: 1, id: Uuid::new_v4(),
        event_type: EVENT_FORM34A_CAPTURED.to_string(),
        payload: json!({ "station_code": "047-0001-001", "constituency_code": "047",
                         "county_code": "47", "registered_voters": 700, "votes_cast": 500,
                         "valid_votes": 490, "rejected_votes": 7, "spoilt_ballots": 3,
                         "candidates": [{"candidate_id":"C001","candidate_name":"Alice","votes":300},
                                        {"candidate_id":"C002","candidate_name":"Bob","votes":190}] }),
        prev_hash: None, hlc: "0".to_string(), recorded_at: Utc::now(),
    }
}

#[test]
fn form34a_apply_captured_event() {
    let stream_id = Uuid::new_v4();
    let mut form = Form34A::new(stream_id);
    form.apply(&make_captured_event(stream_id)).expect("apply should succeed");
    assert_eq!(form.station_code, "047-0001-001");
    assert_eq!(form.registered_voters, 700);
    assert_eq!(form.valid_votes, 490);
    assert_eq!(form.candidates.len(), 2);
    assert_eq!(form.status, Form34AStatus::Captured);
    assert_eq!(form.version, 1);
}

#[test]
fn form34a_snapshot_round_trip() {
    let stream_id = Uuid::new_v4();
    let mut form = Form34A::new(stream_id);
    form.apply(&make_captured_event(stream_id)).unwrap();
    let snapshot = form.snapshot();
    assert_eq!(snapshot.stream_id, stream_id);
    let restored: Form34A = serde_json::from_value(snapshot.state).unwrap();
    assert_eq!(restored.station_code, "047-0001-001");
    assert_eq!(restored.status, Form34AStatus::Captured);
}

#[test]
fn form34a_stream_id_matches() {
    let stream_id = Uuid::new_v4();
    assert_eq!(Form34A::new(stream_id).stream_id(), stream_id);
}
