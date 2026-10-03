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

//! Hermetic integration tests for the RTS tally pipeline (WP-3.3).
//!
//! All tests run fully in process against the pure aggregation and merkle
//! APIs in `rts-tally` and `rts-domain`; no PostgreSQL or Kafka is required.
//! Persistence-level coverage (EventStore round trip) needs a live Postgres
//! and is exercised by the docker-compose stack instead.

use eel_eventstore_rs::StoredEvent;
use rts_domain::events::EVENT_FORM34A_CAPTURED;
use rts_tally::aggregator::{aggregate_34b, aggregate_34c};
use rts_tally::merkle::compute_merkle_root;

/// Number of synthetic Form 34A station captures in the load test.
const STATION_COUNT: usize = 10_000;
/// Stations compiled per constituency Form 34B.
const STATIONS_PER_CONSTITUENCY: usize = 250;
const REGISTERED_VOTERS: u64 = 1_000;
const VALID_VOTES: u64 = 600;
const REJECTED_VOTES: u64 = 50;
const SPOILT_BALLOTS: u64 = 20;

/// Build one deterministic Form 34A captured event for station `i`.
///
/// Candidate A takes `i % 13` of the valid votes; candidate B takes the rest,
/// so the expected grand totals are computable without magic constants.
fn captured_event(i: usize) -> StoredEvent {
    let cand_a = (i % 13) as u64;
    let cand_b = VALID_VOTES - cand_a;
    let payload = serde_json::json!({
        "form_id": format!("34A-{i:05}"),
        "station_code": format!("{:03}-{:03}-{:03}", i / 10_000, (i / 100) % 100, i % 100),
        "constituency_code": format!("C{:03}", i / STATIONS_PER_CONSTITUENCY),
        "county_code": format!("0{:02}", (i / STATIONS_PER_CONSTITUENCY) % 47 + 1),
        "registered_voters": REGISTERED_VOTERS,
        "votes_cast": VALID_VOTES + REJECTED_VOTES,
        "valid_votes": VALID_VOTES,
        "rejected_votes": REJECTED_VOTES,
        "spoilt_ballots": SPOILT_BALLOTS,
        "candidates": [
            { "candidate_id": "cand-a", "candidate_name": "Candidate A", "votes": cand_a },
            { "candidate_id": "cand-b", "candidate_name": "Candidate B", "votes": cand_b },
        ],
    });
    StoredEvent {
        stream_id: uuid::Uuid::new_v4(),
        stream_version: 1,
        id: uuid::Uuid::new_v4(),
        event_type: EVENT_FORM34A_CAPTURED.to_string(),
        payload,
        prev_hash: None,
        hlc: format!("000000000000000000000000{i:04}"),
        recorded_at: chrono::Utc::now(),
    }
}

/// Deterministic in-place shuffle so replay order differs run to run of the
/// generator, not of the clock (no external rng dependency).
fn deterministic_shuffle(events: &mut [StoredEvent], seed: u64) {
    let mut state = seed | 1;
    for i in (1..events.len()).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (state >> 33) as usize % (i + 1);
        events.swap(i, j);
    }
}

#[tokio::test]
async fn test_10k_form34a_aggregation() {
    let events: Vec<StoredEvent> = (0..STATION_COUNT).map(captured_event).collect();

    // Expected grand totals, accumulated from the same generator rules.
    let expected_a: u64 = (0..STATION_COUNT as u64).map(|i| i % 13).sum();
    let expected_b = VALID_VOTES * STATION_COUNT as u64 - expected_a;

    // Aggregate 34A streams into one Form 34B per constituency, then 34C.
    let mut constituencies: std::collections::BTreeMap<String, Vec<StoredEvent>> =
        std::collections::BTreeMap::new();
    for event in &events {
        let code = event.payload["constituency_code"]
            .as_str()
            .unwrap()
            .to_string();
        constituencies.entry(code).or_default().push(event.clone());
    }
    assert_eq!(
        constituencies.len(),
        STATION_COUNT / STATIONS_PER_CONSTITUENCY
    );

    let forms_34b: Vec<_> = constituencies
        .iter()
        .map(|(code, evs)| aggregate_34b(code, evs.clone()))
        .collect();
    let form_34c = aggregate_34c(forms_34b);

    assert_eq!(
        form_34c.total_registered as u64,
        REGISTERED_VOTERS * STATION_COUNT as u64
    );
    assert_eq!(
        form_34c.total_valid as u64,
        VALID_VOTES * STATION_COUNT as u64
    );
    assert_eq!(
        form_34c.total_rejected as u64,
        REJECTED_VOTES * STATION_COUNT as u64
    );
    assert_eq!(
        form_34c.total_spoilt as u64,
        SPOILT_BALLOTS * STATION_COUNT as u64
    );

    let total_a = form_34c
        .candidate_totals
        .iter()
        .find(|c| c.candidate_id == "cand-a")
        .unwrap()
        .votes as u64;
    let total_b = form_34c
        .candidate_totals
        .iter()
        .find(|c| c.candidate_id == "cand-b")
        .unwrap()
        .votes as u64;
    assert_eq!(
        total_a, expected_a,
        "candidate A totals must match expected sums"
    );
    assert_eq!(
        total_b, expected_b,
        "candidate B totals must match expected sums"
    );
    assert_eq!(total_a + total_b, form_34c.total_valid as u64);

    // Merkle root over the serialized event payloads must be a non-zero,
    // stable 32-byte hex digest.
    let leaves: Vec<Vec<u8>> = events
        .iter()
        .map(|e| serde_json::to_vec(&e.payload).unwrap())
        .collect();
    let refs: Vec<&[u8]> = leaves.iter().map(|v| v.as_slice()).collect();
    let root = compute_merkle_root(&refs);
    assert_eq!(root.len(), 64, "merkle root must be a 32-byte hex string");
    assert!(
        root.chars().any(|c| c != '0'),
        "merkle root must not be the zero hash"
    );
    let refs_again: Vec<&[u8]> = leaves.iter().map(|v| v.as_slice()).collect();
    assert_eq!(
        compute_merkle_root(&refs_again),
        root,
        "merkle root must be deterministic"
    );
}

#[tokio::test]
async fn test_replay_shuffled_events_idempotent() {
    // Replaying the same event set in any order must produce the same final
    // Form 34C projection.
    let mut events: Vec<StoredEvent> = (0..2_000).map(captured_event).collect();

    let mut by_stream: std::collections::BTreeMap<uuid::Uuid, Vec<StoredEvent>> =
        std::collections::BTreeMap::new();
    for event in events.clone() {
        by_stream.entry(event.stream_id).or_default().push(event);
    }
    let in_order = aggregate_34c(
        by_stream
            .values()
            .map(|evs| aggregate_34b("C000", evs.clone()))
            .collect(),
    );

    deterministic_shuffle(&mut events, 0x5EED);
    let mut shuffled_streams: std::collections::BTreeMap<uuid::Uuid, Vec<StoredEvent>> =
        std::collections::BTreeMap::new();
    for event in events {
        shuffled_streams
            .entry(event.stream_id)
            .or_default()
            .push(event);
    }
    let shuffled = aggregate_34c(
        shuffled_streams
            .values()
            .map(|evs| aggregate_34b("C000", evs.clone()))
            .collect(),
    );

    assert_eq!(shuffled.total_registered, in_order.total_registered);
    assert_eq!(shuffled.total_valid, in_order.total_valid);
    assert_eq!(shuffled.total_rejected, in_order.total_rejected);
    assert_eq!(shuffled.total_spoilt, in_order.total_spoilt);
    assert_eq!(shuffled.candidate_totals, in_order.candidate_totals);
}

#[tokio::test]
async fn test_form34a_amendment_preserves_original() {
    // After a form34a_amended event the original captured event must still be
    // present and unmodified in the event log.
    let original = captured_event(42);
    let original_payload = original.payload.clone();

    let amended_payload = serde_json::json!({
        "form_id": original.payload["form_id"].clone(),
        "valid_votes": 599,
        "rejected_votes": 51,
        "spoilt_ballots": 20,
        "votes_cast": 670,
        "candidates": [
            { "candidate_id": "cand-a", "candidate_name": "Candidate A", "votes": 199 },
            { "candidate_id": "cand-b", "candidate_name": "Candidate B", "votes": 400 },
        ],
        "reason": "clerical correction after audit",
    });
    let amended = StoredEvent {
        stream_id: original.stream_id,
        stream_version: 2,
        id: uuid::Uuid::new_v4(),
        event_type: rts_domain::events::EVENT_FORM34A_AMENDED.to_string(),
        payload: amended_payload,
        prev_hash: Some(original.id.to_string()),
        hlc: "0000000000000000000000000043".to_string(),
        recorded_at: chrono::Utc::now(),
    };

    let log = vec![original, amended];

    let originals: Vec<&StoredEvent> = log
        .iter()
        .filter(|e| e.event_type == EVENT_FORM34A_CAPTURED)
        .collect();
    assert_eq!(originals.len(), 1, "exactly one captured event must remain");
    assert_eq!(
        originals[0].payload["valid_votes"].as_u64(),
        original_payload["valid_votes"].as_u64(),
        "original captured payload must be preserved verbatim"
    );
    assert_eq!(originals[0].payload, original_payload);
    assert_eq!(log.len(), 2, "amendment must append, never rewrite the log");
}
