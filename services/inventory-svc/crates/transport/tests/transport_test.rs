use chrono::{Duration, Utc};
use transport::domain::{
    assign_security, AssignmentStatus, AssignmentTarget, LoadItem, LoadStatus, ScanMethod,
    SecurityAssignment, TransportError, Vehicle, VehicleLoad, VehicleStatus, VehicleType,
};

fn vehicle(capacity: u64) -> Vehicle {
    Vehicle {
        id: "V-001".into(),
        registration: "KDE 123A".into(),
        vehicle_type: VehicleType::Truck,
        capacity,
        status: VehicleStatus::Available,
    }
}

fn load(vehicle_id: &str) -> VehicleLoad {
    VehicleLoad::new(
        "L-001".into(),
        vehicle_id.into(),
        "KE-022".into(),
        None,
        Utc::now(),
    )
}

fn item(lot_id: &str, quantity: u64, scan_method: ScanMethod) -> LoadItem {
    LoadItem {
        id: format!("I-{lot_id}"),
        lot_id: lot_id.into(),
        quantity,
        scan_method,
        scan_value: "EPC-3008-B94C-0001".into(),
        scanned_by: "officer-9".into(),
        scanned_at: Utc::now(),
    }
}

#[test]
fn loads_a_scanned_ballot_lot_within_capacity() {
    let v = vehicle(10);
    let mut l = load(&v.id);

    l.add_item(&v, item("LOT-1", 4, ScanMethod::Rfid)).unwrap();
    l.add_item(&v, item("LOT-2", 6, ScanMethod::Barcode))
        .unwrap();

    assert_eq!(l.loaded_quantity(), 10);
    assert_eq!(l.items.len(), 2);
}

#[test]
fn rejects_load_exceeding_vehicle_capacity() {
    let v = vehicle(10);
    let mut l = load(&v.id);
    l.add_item(&v, item("LOT-1", 8, ScanMethod::Rfid)).unwrap();

    let err = l
        .add_item(&v, item("LOT-2", 5, ScanMethod::Rfid))
        .unwrap_err();
    assert_eq!(
        err,
        TransportError::CapacityExceeded {
            requested: 13,
            capacity: 10
        }
    );
    assert_eq!(l.items.len(), 1, "failed scan must not be loaded");
}

#[test]
fn rejects_duplicate_lot() {
    let v = vehicle(100);
    let mut l = load(&v.id);
    l.add_item(&v, item("LOT-1", 5, ScanMethod::Barcode))
        .unwrap();

    let err = l
        .add_item(&v, item("LOT-1", 3, ScanMethod::Rfid))
        .unwrap_err();
    assert_eq!(err, TransportError::LotAlreadyLoaded("LOT-1".into()));
}

#[test]
fn rejects_scan_without_value_or_officer() {
    let v = vehicle(100);
    let mut l = load(&v.id);
    let mut blank = item("LOT-1", 1, ScanMethod::Rfid);
    blank.scan_value = " ".into();
    assert_eq!(
        l.add_item(&v, blank).unwrap_err(),
        TransportError::InvalidScan
    );

    let mut no_officer = item("LOT-1", 1, ScanMethod::Rfid);
    no_officer.scanned_by = String::new();
    assert_eq!(
        l.add_item(&v, no_officer).unwrap_err(),
        TransportError::InvalidScan
    );
}

#[test]
fn load_state_machine_dispatch_and_deliver() {
    let v = vehicle(10);
    let mut l = load(&v.id);

    // Cannot dispatch empty.
    assert_eq!(
        l.dispatch(Utc::now()).unwrap_err(),
        TransportError::EmptyLoad
    );

    l.add_item(&v, item("LOT-1", 5, ScanMethod::Rfid)).unwrap();
    l.dispatch(Utc::now()).unwrap();
    assert_eq!(l.status, LoadStatus::Dispatched);
    assert!(l.dispatched_at.is_some());

    // Cannot add items after dispatch.
    let err = l
        .add_item(&v, item("LOT-2", 1, ScanMethod::Rfid))
        .unwrap_err();
    assert!(matches!(err, TransportError::NotLoading(_)));

    l.deliver(Utc::now()).unwrap();
    assert_eq!(l.status, LoadStatus::Delivered);
    assert!(l.delivered_at.is_some());
}

#[test]
fn security_assignment_validates_window_and_prevents_double_booking() {
    let start = Utc::now();
    let end = start + Duration::hours(8);

    let a = assign_security(
        "SA-1".into(),
        "agent-7".into(),
        AssignmentTarget::Vehicle,
        "V-001".into(),
        start,
        end,
        &[],
    )
    .unwrap();
    assert_eq!(a.status, AssignmentStatus::Assigned);

    // Same agent, overlapping window on a different target -> blocked.
    let overlap = start + Duration::hours(4);
    let err = assign_security(
        "SA-2".into(),
        "agent-7".into(),
        AssignmentTarget::Station,
        "STN-022-004".into(),
        overlap,
        overlap + Duration::hours(2),
        std::slice::from_ref(&a),
    )
    .unwrap_err();
    assert!(matches!(err, TransportError::AlreadyAssigned { .. }));

    // Same agent, non-overlapping window -> allowed.
    let later = end + Duration::hours(1);
    let ok = assign_security(
        "SA-3".into(),
        "agent-7".into(),
        AssignmentTarget::Station,
        "STN-022-004".into(),
        later,
        later + Duration::hours(4),
        std::slice::from_ref(&a),
    )
    .unwrap();
    assert_eq!(ok.target, AssignmentTarget::Station);
}

#[test]
fn security_assignment_rejects_inverted_window() {
    let now = Utc::now();
    let err = SecurityAssignment::new(
        "SA-9".into(),
        "agent-1".into(),
        AssignmentTarget::Station,
        "STN-1".into(),
        now + Duration::hours(2),
        now,
    )
    .unwrap_err();
    assert_eq!(err, TransportError::InvalidWindow);
}

#[test]
fn vehicle_is_loadable_only_when_available_or_loading() {
    let mut v = vehicle(10);
    assert!(v.can_be_loaded());

    v.status = VehicleStatus::Dispatched;
    assert!(!v.can_be_loaded());

    v.status = VehicleStatus::Returned;
    assert!(!v.can_be_loaded());
}
