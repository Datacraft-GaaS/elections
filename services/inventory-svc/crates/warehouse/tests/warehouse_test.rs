use chrono::{Duration, Utc};
use warehouse::domain::{
    Incident, IncidentSeverity, IncidentStatus, IncidentType, ItemType, Pallet, PalletItem,
    PalletStatus, TransferStatus, Warehouse, WarehouseError, WarehouseTier, WarehouseTransfer,
};

fn warehouse(id: &str, tier: WarehouseTier, lat: f64, lon: f64) -> Warehouse {
    Warehouse::new(
        id.into(),
        format!("{id}-code"),
        format!("{id} name"),
        tier,
        lat,
        lon,
        500.0,
    )
    .unwrap()
}

#[test]
fn warehouse_geofence_contains_point() {
    let w = warehouse("WH-1", WarehouseTier::County, -1.2921, 36.8219);
    assert!(w.contains(-1.2921, 36.8219));
    assert!(w.contains(-1.2940, 36.8220));
    // Far outside the 500 m radius.
    assert!(!w.contains(-1.4000, 36.9000));
}

#[test]
fn warehouse_rejects_non_positive_geofence() {
    let err = Warehouse::new(
        "WH-2".into(),
        "WH-2-code".into(),
        "WH-2 name".into(),
        WarehouseTier::Master,
        -1.2921,
        36.8219,
        0.0,
    )
    .unwrap_err();
    assert_eq!(err, WarehouseError::InvalidGeofence { radius: 0.0 });
}

#[test]
fn transfer_cannot_start_and_end_at_same_warehouse() {
    let err = WarehouseTransfer::new(
        "T-1".into(),
        "WH-1".into(),
        "WH-1".into(),
        None,
        vec!["P-1".into()],
        Utc::now(),
    )
    .unwrap_err();
    assert_eq!(err, WarehouseError::NoSelfTransfer);
}

#[test]
fn transfer_depart_arrive_and_reconcile() {
    let mut t = WarehouseTransfer::new(
        "T-1".into(),
        "WH-M".into(),
        "WH-C".into(),
        Some("V-001".into()),
        vec!["P-1".into(), "P-2".into(), "P-3".into()],
        Utc::now(),
    )
    .unwrap();

    // Cannot arrive before departure.
    assert!(matches!(
        t.arrive(vec!["P-1".into()], Utc::now()),
        Err(WarehouseError::InvalidTransition { .. })
    ));

    t.depart(Utc::now()).unwrap();
    assert_eq!(t.status, TransferStatus::Departed);

    // Two of three pallets arrive, plus an unexpected one.
    t.arrive(vec!["P-1".into(), "P-3".into(), "P-9".into()], Utc::now())
        .unwrap();
    assert_eq!(t.status, TransferStatus::Arrived);
    assert_eq!(t.missing_pallets(), vec!["P-2"]);
    assert_eq!(t.extra_pallets(), vec!["P-9"]);

    // Cannot depart again after arrival.
    assert!(matches!(
        t.depart(Utc::now()),
        Err(WarehouseError::InvalidTransition { .. })
    ));
}

#[test]
fn transfer_cancel_only_from_created() {
    let mut t = WarehouseTransfer::new(
        "T-2".into(),
        "WH-M".into(),
        "WH-C".into(),
        None,
        vec!["P-1".into()],
        Utc::now(),
    )
    .unwrap();
    t.cancel().unwrap();
    assert_eq!(t.status, TransferStatus::Cancelled);
    assert!(matches!(
        t.depart(Utc::now()),
        Err(WarehouseError::InvalidTransition { .. })
    ));
}

#[test]
fn pallet_lifecycle_receipt_to_polling_station() {
    let mut p = Pallet::receive(
        "P-1".into(),
        "KE27-PAL-0001".into(),
        "WH-M".into(),
        Utc::now(),
    );
    assert_eq!(p.status, PalletStatus::Received);

    p.store().unwrap();
    assert_eq!(p.status, PalletStatus::InStorage);

    p.start_transfer().unwrap();
    assert_eq!(p.status, PalletStatus::InTransit);

    p.complete_transfer("WH-C".into()).unwrap();
    assert_eq!(p.status, PalletStatus::InStorage);
    assert_eq!(p.warehouse_id, "WH-C");

    // Break bulk into ballot boxes.
    p.break_bulk(vec![
        PalletItem {
            id: "I-1".into(),
            item_type: ItemType::BallotBox,
            reference_id: "BOX-001".into(),
            quantity: 20,
        },
        PalletItem {
            id: "I-2".into(),
            item_type: ItemType::BallotLot,
            reference_id: "LOT-7F3K".into(),
            quantity: 1,
        },
    ])
    .unwrap();
    assert_eq!(p.status, PalletStatus::BreakBulk);
    assert_eq!(p.items.len(), 2);

    p.dispatch().unwrap();
    assert_eq!(p.status, PalletStatus::Dispatched);

    p.mark_used("STN-022-004".into()).unwrap();
    assert_eq!(p.status, PalletStatus::Used);
    assert_eq!(p.used_at_station.as_deref(), Some("STN-022-004"));
}

#[test]
fn pallet_rejects_wrong_transitions_and_empty_break_bulk() {
    let mut p = Pallet::receive(
        "P-2".into(),
        "KE27-PAL-0002".into(),
        "WH-M".into(),
        Utc::now(),
    );

    // Cannot break bulk straight from received.
    assert!(matches!(
        p.break_bulk(vec![PalletItem {
            id: "I-1".into(),
            item_type: ItemType::BallotBox,
            reference_id: "BOX-1".into(),
            quantity: 1,
        }]),
        Err(WarehouseError::InvalidTransition { .. })
    ));

    p.store().unwrap();
    assert!(matches!(
        p.break_bulk(vec![]),
        Err(WarehouseError::EmptyBreakBulk)
    ));
}

#[test]
fn incident_lifecycle_and_notes() {
    let mut i = Incident::new(
        "INC-1".into(),
        Some("WH-C".into()),
        Some("P-1".into()),
        None,
        IncidentType::Tamper,
        IncidentSeverity::P0,
        "Seal on pallet P-1 broken outside a custody event".into(),
        "officer-9".into(),
        Utc::now(),
    )
    .unwrap();
    assert_eq!(i.status, IncidentStatus::Open);

    i.investigate().unwrap();
    assert_eq!(i.status, IncidentStatus::Investigating);

    i.add_note(
        "N-1".into(),
        "officer-9".into(),
        "Footage shows a forklift strike; no malice.".into(),
        Utc::now() + Duration::minutes(10),
    );
    assert_eq!(i.notes.len(), 1);

    i.resolve(Utc::now()).unwrap();
    assert_eq!(i.status, IncidentStatus::Resolved);
    assert!(i.resolved_at.is_some());

    // Cannot resolve twice.
    assert!(matches!(
        i.resolve(Utc::now()),
        Err(WarehouseError::AlreadyResolved)
    ));
}

#[test]
fn incident_rejects_empty_summary() {
    let err = Incident::new(
        "INC-2".into(),
        None,
        None,
        None,
        IncidentType::Other,
        IncidentSeverity::P3,
        "   ".into(),
        "officer-9".into(),
        Utc::now(),
    )
    .unwrap_err();
    assert_eq!(err, WarehouseError::EmptySummary);
}
