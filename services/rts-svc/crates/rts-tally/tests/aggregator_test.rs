use chrono::Utc;
use eel_eventstore_rs::StoredEvent;
use eel_trace_rs::trace;
use rts_domain::events::EVENT_FORM34A_CAPTURED;
use rts_tally::aggregator::{aggregate_34b, aggregate_34c};
use serde_json::json;
use uuid::Uuid;

fn make_event(station: &str, valid: u32, c001: u32) -> StoredEvent {
    StoredEvent {
        stream_id: Uuid::new_v4(),
        stream_version: 1,
        id: Uuid::new_v4(),
        event_type: EVENT_FORM34A_CAPTURED.to_string(),
        payload: json!({ "station_code": station, "constituency_code": "047", "county_code": "47",
                         "registered_voters": 700, "votes_cast": valid + 10, "valid_votes": valid,
                         "rejected_votes": 7, "spoilt_ballots": 3,
                         "candidates": [{"candidate_id":"C001","candidate_name":"Alice","votes":c001},
                                        {"candidate_id":"C002","candidate_name":"Bob","votes":valid-c001}] }),
        prev_hash: None,
        hlc: "0".to_string(),
        recorded_at: Utc::now(),
    }
}

#[trace("FR-M10-020")]
#[test]
fn test_34a_to_34b_aggregation() {
    let events = vec![
        make_event("047-0001", 100, 60),
        make_event("047-0002", 200, 120),
        make_event("047-0003", 150, 90),
    ];
    let form34b = aggregate_34b("047", events);
    assert_eq!(form34b.constituency_code, "047");
    assert_eq!(form34b.total_valid, 450);
    assert_eq!(form34b.compiled_stations.len(), 3);
    let c001 = form34b
        .candidate_totals
        .iter()
        .find(|c| c.candidate_id == "C001")
        .unwrap();
    assert_eq!(c001.votes, 270);
}

#[trace("FR-M10-020")]
#[test]
fn test_34b_to_34c_aggregation() {
    let form34b_047 = aggregate_34b("047", vec![make_event("047-0001", 100, 60)]);
    let form34b_048 = aggregate_34b("048", vec![make_event("048-0001", 200, 120)]);
    let form34c = aggregate_34c(vec![form34b_047, form34b_048]);
    assert_eq!(form34c.compiled_constituencies.len(), 2);
    assert_eq!(form34c.total_valid, 300);
    let c001 = form34c
        .candidate_totals
        .iter()
        .find(|c| c.candidate_id == "C001")
        .unwrap();
    assert_eq!(c001.votes, 180);
}
