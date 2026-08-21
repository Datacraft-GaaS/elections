use eel_queue_pipeline::models::SignatureFusionInput;
use eel_queue_pipeline::pipeline::{
    check_disenfranchisement, check_wait_breach, fuse_snapshot, recommend_s14a,
};

#[test]
fn integration_fuse_and_events_normal() {
    let input = SignatureFusionInput {
        kit_throughput_per_hour: 120,
        clerk_report_count: 80,
        cv_headcount: 90,
        qr_scan_count: 85,
    };
    let snap = fuse_snapshot("STN-INT-001", input);
    assert_eq!(snap.station_code, "STN-INT-001");
    assert!(!snap.disenfranchisement_flag);
    assert!(check_wait_breach(&snap, 999).is_none());
    assert!(check_disenfranchisement(&snap).is_none());
    assert!(recommend_s14a(&snap).is_none());
}

#[test]
fn integration_fuse_and_events_breach() {
    let input = SignatureFusionInput {
        kit_throughput_per_hour: 5,
        clerk_report_count: 200,
        cv_headcount: 210,
        qr_scan_count: 195,
    };
    let snap = fuse_snapshot("STN-INT-002", input);
    assert!(snap.disenfranchisement_flag);
    assert!(snap.s14a_recommended);
    let breach = check_wait_breach(&snap, 30);
    assert!(breach.is_some());
    let dis = check_disenfranchisement(&snap);
    assert!(dis.is_some());
    let s14a = recommend_s14a(&snap);
    assert!(s14a.is_some());
    assert!(s14a.unwrap().hsm_pending);
}
