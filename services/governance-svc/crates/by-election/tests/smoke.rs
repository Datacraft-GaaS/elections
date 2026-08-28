#[tokio::test]
async fn router_smoke() {
    let state = eel_governance_by_election::AppState::new();
    let app = eel_governance_by_election::router::build_router(state);
    drop(app);
}

#[test]
fn short_notice_detection() {
    use eel_governance_by_election::models::ByElection;
    let mut be = ByElection::default();
    be.notice_date = "2025-01-01".to_string();
    // 20 days: short notice
    be.election_date = "2025-01-21".to_string();
    assert!(be.is_short_notice());
    // 21 days: not short notice (exactly at limit)
    be.election_date = "2025-01-22".to_string();
    assert!(!be.is_short_notice());
    // 30 days: not short notice
    be.election_date = "2025-01-31".to_string();
    assert!(!be.is_short_notice());
}

#[test]
fn cost_report_total_validation() {
    use eel_governance_by_election::cost_report::build_cost_report;
    let report = build_cost_report("be-1".into(), 100, 200, 50, 30, 20, 10).expect("valid");
    assert_eq!(report.total_kes, 410);
    assert!(report.validate_total().is_ok());
}

#[test]
fn cost_report_all_seven_line_items_non_zero() {
    use eel_governance_by_election::cost_report::build_cost_report;
    let report = build_cost_report("be-2".into(), 1, 2, 3, 4, 5, 6).expect("valid");
    assert!(report.kit_days_kes > 0);
    assert!(report.staff_days_kes > 0);
    assert!(report.transport_kes > 0);
    assert!(report.printing_kes > 0);
    assert!(report.venue_kes > 0);
    assert!(report.other_kes > 0);
    assert_eq!(report.total_kes, 21);
}

#[test]
fn kit_plan_reserves_one_kit_per_station() {
    use eel_governance_by_election::kit_plan::generate_kit_plan;
    let plan = generate_kit_plan("be-3".into(), vec!["S1".into(), "S2".into(), "S3".into()]);
    assert_eq!(plan.station_assignments.len(), 3);
    assert_eq!(plan.kit_ids.len(), 3);
    // All unique kit IDs
    let unique: std::collections::HashSet<_> = plan.kit_ids.iter().collect();
    assert_eq!(unique.len(), 3);
}

#[test]
fn concurrency_limit_constant() {
    use eel_governance_by_election::models::MAX_CONCURRENT_BY_ELECTIONS;
    assert_eq!(MAX_CONCURRENT_BY_ELECTIONS, 10);
}
