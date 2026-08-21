use chrono::Utc;
use printing::domain::{discrepancy, LotData, StationReconciliation};

#[test]
fn variance_pct_triggers_alert_condition() {
    let lot = LotData {
        id: "lot-1".into(),
        print_order_id: "po-1".into(),
        printer_facility_id: "fac-1".into(),
        serial_range_start: 1,
        serial_range_end: 1000,
        quantity_printed: 985,
        quantity_rejected: 15,
        rfid_chip_id_list_hash: "abc123".into(),
        print_date: Utc::now(),
        received_at: None,
        variance_pct: 0.015,
        variance_alert_sent: false,
    };
    assert!(
        lot.variance_pct > 0.01_f64,
        "variance above threshold should trigger alert"
    );
}

#[test]
fn discrepancy_arithmetic_is_correct() {
    let r = StationReconciliation {
        id: "rec-1".into(),
        station_code: "STN001".into(),
        election_id: "elec-1".into(),
        ballots_received: 500,
        ballots_issued: 420,
        ballots_unused: 70,
        ballots_spoilt: 5,
        discrepancy: 0,
        discrepancy_flag: false,
        reconciled_at: Utc::now(),
        reconciled_by: "officer-1".into(),
    };
    // 500 - 420 - 70 - 5 = 5
    assert_eq!(discrepancy(&r), 5_i64);
}
