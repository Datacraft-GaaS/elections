//! Integration + unit tests for telemetry-query.
//!
//! All tests are pure unit tests (no live DB); they exercise DSL round-trip,
//! validator correctness, planner ordering, executor row-cap logic, preset
//! builder, NL bridge mock, and evidence bundle sha256 stability.

use chrono::{Duration, Utc};
use eel_ecc_telemetry_query::{
    dsl::{
        Aggregation, AggregationOp, Field, GeoPolygon, OrderBy, OrderDir, SignalClass,
        TelemetryQuery, TimeRange,
    },
    evidence_bundle::EvidenceBundleBuilder,
    executor::ROW_CAP,
    nl_bridge::{MockBridge, NlQueryBridge},
    planner::{CostModelPlanner, ScanOrder},
    preset::PresetLibrary,
    validator::{TelemetryQueryValidator, ValidationError},
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn short_range() -> TimeRange {
    let now = Utc::now();
    TimeRange {
        from: now - Duration::hours(1),
        to: now,
    }
}

fn basic_query() -> TelemetryQuery {
    TelemetryQuery {
        device_ids: vec![Uuid::new_v4()],
        signal_classes: vec![SignalClass::Location],
        time_range: short_range(),
        geo_polygon: None,
        aggregations: vec![],
        group_by: vec![],
        limit: 100,
        order_by: vec![],
    }
}

// ---------------------------------------------------------------------------
// DSL round-trip tests
// ---------------------------------------------------------------------------

#[test]
fn test_dsl_roundtrip_basic() {
    let q = basic_query();
    let json = serde_json::to_string(&q).expect("serialize");
    let q2: TelemetryQuery = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(q.limit, q2.limit);
    assert_eq!(q.device_ids, q2.device_ids);
}

#[test]
fn test_dsl_roundtrip_all_signal_classes() {
    let classes = vec![
        SignalClass::Location,
        SignalClass::Time,
        SignalClass::Activity,
        SignalClass::ProximalRfid,
        SignalClass::Movements,
        SignalClass::LoginLogout,
        SignalClass::Reads,
        SignalClass::Writes,
        SignalClass::Battery,
        SignalClass::LocksUnlocks,
        SignalClass::PhysicalDrops,
        SignalClass::Network,
        SignalClass::Attestation,
    ];
    assert_eq!(classes.len(), 13);
    for sc in &classes {
        let json = serde_json::to_string(sc).unwrap();
        let sc2: SignalClass = serde_json::from_str(&json).unwrap();
        assert_eq!(sc, &sc2);
    }
}

#[test]
fn test_dsl_default_limit() {
    let json = r#"{"time_range":{"from":"2026-01-01T00:00:00Z","to":"2026-01-02T00:00:00Z"}}"#;
    let q: TelemetryQuery = serde_json::from_str(json).expect("deserialize");
    assert_eq!(q.limit, 1_000);
}

#[test]
fn test_dsl_all_aggregation_ops_roundtrip() {
    let ops = vec![
        AggregationOp::Count,
        AggregationOp::Sum,
        AggregationOp::Avg,
        AggregationOp::Min,
        AggregationOp::Max,
        AggregationOp::Distinct,
    ];
    for op in &ops {
        let json = serde_json::to_string(op).unwrap();
        let op2: AggregationOp = serde_json::from_str(&json).unwrap();
        assert_eq!(op, &op2);
    }
}

#[test]
fn test_dsl_geo_polygon_roundtrip() {
    let poly = GeoPolygon {
        coordinates: vec![[36.8, -1.3], [37.0, -1.3], [37.0, -1.1], [36.8, -1.1]],
    };
    let json = serde_json::to_string(&poly).unwrap();
    let poly2: GeoPolygon = serde_json::from_str(&json).unwrap();
    assert_eq!(poly.coordinates.len(), poly2.coordinates.len());
}

// ---------------------------------------------------------------------------
// Validator tests
// ---------------------------------------------------------------------------

#[test]
fn test_validator_accepts_valid_query() {
    let q = basic_query();
    assert!(TelemetryQueryValidator::validate(&q).is_ok());
}

#[test]
fn test_validator_rejects_inverted_time_range() {
    let now = Utc::now();
    let mut q = basic_query();
    q.time_range = TimeRange {
        from: now,
        to: now - Duration::hours(1),
    };
    let err = TelemetryQueryValidator::validate(&q).unwrap_err();
    match err {
        ValidationError::Invalid(diags) => {
            assert!(diags.iter().any(|d| d.code == "TIME_RANGE_INVERTED"));
        }
    }
}

#[test]
fn test_validator_rejects_time_range_too_wide() {
    let now = Utc::now();
    let mut q = basic_query();
    q.time_range = TimeRange {
        from: now - Duration::days(91),
        to: now,
    };
    let err = TelemetryQueryValidator::validate(&q).unwrap_err();
    match err {
        ValidationError::Invalid(diags) => {
            assert!(diags.iter().any(|d| d.code == "TIME_RANGE_TOO_WIDE"));
        }
    }
}

#[test]
fn test_validator_rejects_group_by_without_aggregation() {
    let mut q = basic_query();
    q.group_by = vec![Field::DeviceId];
    q.aggregations = vec![];
    let err = TelemetryQueryValidator::validate(&q).unwrap_err();
    match err {
        ValidationError::Invalid(diags) => {
            assert!(diags.iter().any(|d| d.code == "GROUP_BY_WITHOUT_AGG"));
        }
    }
}

#[test]
fn test_validator_rejects_geo_polygon_too_many_vertices() {
    let mut q = basic_query();
    q.geo_polygon = Some(GeoPolygon {
        coordinates: vec![[0.0, 0.0]; 101],
    });
    let err = TelemetryQueryValidator::validate(&q).unwrap_err();
    match err {
        ValidationError::Invalid(diags) => {
            assert!(diags.iter().any(|d| d.code == "GEO_POLYGON_TOO_MANY_VERTICES"));
        }
    }
}

#[test]
fn test_validator_rejects_geo_polygon_too_few_vertices() {
    let mut q = basic_query();
    q.geo_polygon = Some(GeoPolygon {
        coordinates: vec![[0.0, 0.0], [1.0, 0.0]],
    });
    let err = TelemetryQueryValidator::validate(&q).unwrap_err();
    match err {
        ValidationError::Invalid(diags) => {
            assert!(diags.iter().any(|d| d.code == "GEO_POLYGON_TOO_FEW_VERTICES"));
        }
    }
}

#[test]
fn test_validator_rejects_limit_zero() {
    let mut q = basic_query();
    q.limit = 0;
    let err = TelemetryQueryValidator::validate(&q).unwrap_err();
    match err {
        ValidationError::Invalid(diags) => {
            assert!(diags.iter().any(|d| d.code == "LIMIT_ZERO"));
        }
    }
}

#[test]
fn test_validator_rejects_limit_over_cap() {
    let mut q = basic_query();
    q.limit = 100_001;
    let err = TelemetryQueryValidator::validate(&q).unwrap_err();
    match err {
        ValidationError::Invalid(diags) => {
            assert!(diags.iter().any(|d| d.code == "LIMIT_EXCEEDS_CAP"));
        }
    }
}

#[test]
fn test_validator_accepts_valid_aggregation_with_group_by() {
    let mut q = basic_query();
    q.aggregations = vec![Aggregation {
        op: AggregationOp::Count,
        field: Field::DeviceId,
        alias: None,
    }];
    q.group_by = vec![Field::DeviceId];
    assert!(TelemetryQueryValidator::validate(&q).is_ok());
}

// ---------------------------------------------------------------------------
// Planner tests
// ---------------------------------------------------------------------------

#[test]
fn test_planner_single_device_short_range_device_first() {
    let q = basic_query();
    let plan = CostModelPlanner::plan(&q);
    assert!(matches!(plan.scan_order, ScanOrder::DeviceThenTime));
    assert!(!plan.partitions.is_empty());
}

#[test]
fn test_planner_no_devices_is_partition_scan() {
    let mut q = basic_query();
    q.device_ids = vec![];
    let plan = CostModelPlanner::plan(&q);
    assert!(matches!(plan.scan_order, ScanOrder::PartitionScan));
}

#[test]
fn test_planner_many_devices_time_then_device() {
    let mut q = basic_query();
    q.device_ids = (0..15).map(|_| Uuid::new_v4()).collect();
    let plan = CostModelPlanner::plan(&q);
    assert!(matches!(plan.scan_order, ScanOrder::TimeThenDevice));
}

#[test]
fn test_planner_enumerates_correct_partitions() {
    let q = TelemetryQuery {
        device_ids: vec![],
        signal_classes: vec![],
        time_range: TimeRange {
            from: "2026-03-15T00:00:00Z".parse().unwrap(),
            to: "2026-05-10T00:00:00Z".parse().unwrap(),
        },
        geo_polygon: None,
        aggregations: vec![],
        group_by: vec![],
        limit: 1000,
        order_by: vec![],
    };
    let plan = CostModelPlanner::plan(&q);
    let names: Vec<String> = plan.partitions.iter().map(|p| p.0.clone()).collect();
    assert!(names.contains(&"2026-03".to_string()));
    assert!(names.contains(&"2026-04".to_string()));
    assert!(names.contains(&"2026-05".to_string()));
}

// ---------------------------------------------------------------------------
// Executor row cap test (pure logic)
// ---------------------------------------------------------------------------

#[test]
fn test_row_cap_constant() {
    assert_eq!(ROW_CAP, 5_000_000);
}

// ---------------------------------------------------------------------------
// Preset library tests
// ---------------------------------------------------------------------------

#[test]
fn test_preset_list_has_10_presets() {
    let presets = PresetLibrary::list();
    assert_eq!(presets.len(), 10);
}

#[test]
fn test_preset_drops_by_county_builds() {
    let mut params = serde_json::Map::new();
    params.insert("county".into(), serde_json::json!("Nairobi"));
    params.insert("hours".into(), serde_json::json!(48));
    let result = PresetLibrary::build("drops_by_county", &params);
    assert!(result.is_ok(), "drops_by_county preset failed: {:?}", result.err());
    let (q, desc) = result.unwrap();
    assert!(!q.signal_classes.is_empty());
    assert!(desc.contains("Nairobi") || desc.contains("drop"));
}

#[test]
fn test_preset_officer_activity_builds() {
    let now = Utc::now();
    let mut params = serde_json::Map::new();
    params.insert("officer_id".into(), serde_json::json!("4432"));
    params.insert("from".into(), serde_json::json!((now - Duration::hours(24)).to_rfc3339()));
    params.insert("to".into(), serde_json::json!(now.to_rfc3339()));
    let (q, _desc) = PresetLibrary::build("officer_activity", &params).unwrap();
    assert!(q.aggregations.len() >= 2);
}

#[test]
fn test_preset_attestation_gap_builds() {
    let now = Utc::now();
    let mut params = serde_json::Map::new();
    params.insert("from".into(), serde_json::json!((now - Duration::hours(24)).to_rfc3339()));
    params.insert("to".into(), serde_json::json!(now.to_rfc3339()));
    let (q, _) = PresetLibrary::build("attestation_gap", &params).unwrap();
    assert!(q.signal_classes.contains(&SignalClass::Attestation));
}

#[test]
fn test_preset_unknown_returns_error() {
    let params = serde_json::Map::new();
    assert!(PresetLibrary::build("nonexistent_preset", &params).is_err());
}

#[test]
fn test_preset_battery_low_fleet_builds() {
    let now = Utc::now();
    let mut params = serde_json::Map::new();
    params.insert("threshold_pct".into(), serde_json::json!(15.0));
    params.insert("county".into(), serde_json::json!("Mombasa"));
    params.insert("from".into(), serde_json::json!((now - Duration::hours(4)).to_rfc3339()));
    params.insert("to".into(), serde_json::json!(now.to_rfc3339()));
    let (q, _) = PresetLibrary::build("battery_low_fleet", &params).unwrap();
    assert!(q.signal_classes.contains(&SignalClass::Battery));
}

#[test]
fn test_preset_coordinated_logouts_builds() {
    let now = Utc::now();
    let mut params = serde_json::Map::new();
    params.insert("station_id".into(), serde_json::json!("STN-001"));
    params.insert("from".into(), serde_json::json!((now - Duration::hours(4)).to_rfc3339()));
    params.insert("to".into(), serde_json::json!(now.to_rfc3339()));
    let (q, _) = PresetLibrary::build("coordinated_logouts", &params).unwrap();
    assert!(q.signal_classes.contains(&SignalClass::LoginLogout));
}

#[test]
fn test_all_presets_produce_valid_queries() {
    let now = Utc::now();
    let from = (now - Duration::hours(24)).to_rfc3339();
    let to = now.to_rfc3339();

    let mut common = serde_json::Map::new();
    common.insert("from".into(), serde_json::json!(from));
    common.insert("to".into(), serde_json::json!(to));
    common.insert("county".into(), serde_json::json!("Nairobi"));
    common.insert("station_id".into(), serde_json::json!("STN-001"));
    common.insert("officer_id".into(), serde_json::json!("OFF-001"));
    common.insert("threshold_pct".into(), serde_json::json!(15));
    common.insert("hours".into(), serde_json::json!(24));
    common.insert("sla_minutes".into(), serde_json::json!(30));
    common.insert("radius_m".into(), serde_json::json!(500));

    for descriptor in PresetLibrary::list() {
        let result = PresetLibrary::build(&descriptor.name, &common);
        assert!(
            result.is_ok(),
            "preset {} failed: {:?}",
            descriptor.name,
            result.err()
        );
        let (q, _) = result.unwrap();
        let validation = TelemetryQueryValidator::validate(&q);
        assert!(
            validation.is_ok(),
            "preset {} produced invalid query: {:?}",
            descriptor.name,
            validation.err()
        );
    }
}

// ---------------------------------------------------------------------------
// NL bridge mock tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_mock_bridge_happy_path() {
    let bridge = MockBridge;
    let result = bridge
        .translate("Show me all drops in Nairobi in the last 24 hours", "svid://test")
        .await;
    assert!(result.is_ok());
    let proposed = result.unwrap();
    assert!(!proposed.explanation.is_empty());
    assert!(proposed.confidence > 0.0);
    assert_eq!(proposed.original_text, "Show me all drops in Nairobi in the last 24 hours");
}

#[tokio::test]
async fn test_mock_bridge_returns_valid_dsl() {
    let bridge = MockBridge;
    let proposed = bridge
        .translate("officer sessions for badge 4432", "svid://analyst")
        .await
        .unwrap();
    let validation = TelemetryQueryValidator::validate(&proposed.query);
    assert!(validation.is_ok(), "mock bridge returned invalid DSL: {:?}", validation.err());
}

#[tokio::test]
async fn test_mock_bridge_empty_actor_still_returns() {
    let bridge = MockBridge;
    let result = bridge.translate("any query", "").await;
    // MockBridge doesn't check the actor; it always succeeds
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Evidence bundle tests
// ---------------------------------------------------------------------------

#[test]
fn test_evidence_bundle_sha256_is_stable_for_same_input() {
    let rows = vec![
        serde_json::json!({"device_id": "d1", "signal_class": "location", "occurred_at": "2026-01-01T00:00:00Z"}),
        serde_json::json!({"device_id": "d2", "signal_class": "battery", "occurred_at": "2026-01-01T00:01:00Z"}),
    ];

    let q = basic_query();
    let builder1 = EvidenceBundleBuilder {
        query_id: "test-query-id".into(),
        actor_svid: "svid://analyst".into(),
        query: q.clone(),
        rows: rows.clone(),
        executed_at: Utc::now(),
    };

    let bundle1 = builder1.build().expect("build bundle 1");

    let builder2 = EvidenceBundleBuilder {
        query_id: "test-query-id".into(),
        actor_svid: "svid://analyst".into(),
        query: q.clone(),
        rows: rows.clone(),
        executed_at: Utc::now(),
    };

    let bundle2 = builder2.build().expect("build bundle 2");

    // CSV sha256 should be identical for identical row content
    assert_eq!(bundle1.manifest.csv_sha256, bundle2.manifest.csv_sha256);
}

#[test]
fn test_evidence_bundle_has_nonempty_hsm_signature() {
    let q = basic_query();
    let builder = EvidenceBundleBuilder {
        query_id: "q1".into(),
        actor_svid: "svid://admin".into(),
        query: q,
        rows: vec![],
        executed_at: Utc::now(),
    };
    let bundle = builder.build().expect("build");
    assert!(!bundle.manifest.hsm_signature.is_empty());
}

#[test]
fn test_evidence_bundle_manifest_sha256_nonempty() {
    let q = basic_query();
    let builder = EvidenceBundleBuilder {
        query_id: "q2".into(),
        actor_svid: "svid://admin".into(),
        query: q,
        rows: vec![serde_json::json!({"col": "val"})],
        executed_at: Utc::now(),
    };
    let bundle = builder.build().expect("build");
    assert_eq!(bundle.manifest.manifest_sha256.len(), 64); // hex sha256
}

#[test]
fn test_evidence_bundle_files_exist() {
    let q = basic_query();
    let builder = EvidenceBundleBuilder {
        query_id: "q3".into(),
        actor_svid: "svid://admin".into(),
        query: q,
        rows: vec![serde_json::json!({"device_id": "abc", "signal_class": "battery"})],
        executed_at: Utc::now(),
    };
    let bundle = builder.build().expect("build");
    assert!(bundle.pdf_path.exists());
    assert!(bundle.csv_path.exists());
    assert!(bundle.parquet_path.exists());
    assert!(bundle.manifest_path.exists());
}
