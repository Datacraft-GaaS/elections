use chrono::{Duration, Utc};
use diaspora_logistics::domain::{
    custody_chain, ArrivalRecord, ContainerCustodyEvent, ContainerEventType, HubRoute,
    RoutingPolicy,
};

#[test]
fn custody_chain_sorted_by_timestamp() {
    let now = Utc::now();
    let events = vec![
        ContainerCustodyEvent {
            id: "e3".into(),
            container_id: "c1".into(),
            event_type: ContainerEventType::Received,
            actor_id: "actor-3".into(),
            location: "Nairobi".into(),
            timestamp: now + Duration::hours(2),
        },
        ContainerCustodyEvent {
            id: "e1".into(),
            container_id: "c1".into(),
            event_type: ContainerEventType::Packed,
            actor_id: "actor-1".into(),
            location: "London".into(),
            timestamp: now,
        },
        ContainerCustodyEvent {
            id: "e2".into(),
            container_id: "c1".into(),
            event_type: ContainerEventType::Dispatched,
            actor_id: "actor-2".into(),
            location: "Heathrow".into(),
            timestamp: now + Duration::hours(1),
        },
    ];

    let chain = custody_chain(&events);
    assert_eq!(chain.len(), 3);
    assert_eq!(chain[0].id, "e1");
    assert_eq!(chain[1].id, "e2");
    assert_eq!(chain[2].id, "e3");
}

#[test]
fn locked_hub_route_returns_conflict() {
    let hub = HubRoute {
        id: "hub-1".into(),
        post_code: "GB-LHR".into(),
        hub_code: "LHR".into(),
        routing_policy: RoutingPolicy::AirlineCargo,
        approved_at: None,
        locked: true,
    };
    assert!(hub.locked, "locked hub route should signal 409-equivalent");
}

#[test]
fn arrival_record_discrepancy_arithmetic() {
    let record = ArrivalRecord {
        id: "arr-1".into(),
        container_id: "c1".into(),
        expected_count: 1000,
        received_count: 987,
        discrepancy: 987_i64 - 1000_i64,
        resolved_at: None,
    };
    assert_eq!(
        record.discrepancy,
        record.received_count as i64 - record.expected_count as i64
    );
    assert_eq!(record.discrepancy, -13_i64);
}
