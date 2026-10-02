//! Integration tests for eel-ai-platform-telemetry-analytics.
//! 40+ tests covering all detectors, pipeline, rule registry, backtest, dispatcher, and API.

use chrono::Utc;
use eel_ai_platform_telemetry_analytics::{
    backtest::{run_backtest, LabelledAnomaly},
    detector::{Alert, AlertSeverity, TelemetrySignal},
    detectors::{
        battery_starvation::{BatteryStarvationClusterDetector, BatteryStarvationConfig},
        coordinated::{CoordinatedConfig, CoordinatedDeviceAnomalyDetector},
        drop::{DropAtScaleDetector, DropDetectorConfig},
        geo_fence::{GeoFenceBreachDetector, GeoFenceConfig, StationFence},
        login_time::{LoginBaseline, LoginTimeAnomalyDetector, LoginTimeConfig},
        read_heavy::{ReadHeavyAnomalyDetector, ReadHeavyConfig},
        rfid_absence::{RfidAbsenceAnomalyDetector, RfidAbsenceConfig},
        session_hijack::{SessionHijackConfig, SessionHijackDetector},
        ueba::{UebaConfig, UebaDetector},
    },
    dispatch::{AlertDispatcher, DispatchConfig},
    rule_registry::{RuleEntry, RuleRegistry},
    Detector,
};
use serde_json::json;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

// ========== Helpers ==========

fn sig(
    id: &str,
    device_id: &str,
    signal_type: &str,
    payload: serde_json::Value,
) -> TelemetrySignal {
    TelemetrySignal {
        id: id.to_string(),
        device_id: device_id.to_string(),
        signal_type: signal_type.to_string(),
        payload,
        received_at: Utc::now(),
    }
}

fn sig_at(
    id: &str,
    device_id: &str,
    signal_type: &str,
    payload: serde_json::Value,
    offset_secs: i64,
) -> TelemetrySignal {
    TelemetrySignal {
        id: id.to_string(),
        device_id: device_id.to_string(),
        signal_type: signal_type.to_string(),
        payload,
        received_at: Utc::now() + chrono::Duration::seconds(offset_secs),
    }
}

fn rules_yaml_path() -> String {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../config/telemetry-rules.yaml"
    )
    .to_string()
}

// ========== DropAtScaleDetector ==========

#[tokio::test]
async fn drop_happy_path_critical_during_voting() {
    let det = DropAtScaleDetector::new(DropDetectorConfig::default());
    let signals = vec![sig(
        "s1",
        "dev1",
        "drop",
        json!({ "accel_g": 5.0, "custody_state": "station_open_voting_active", "station_id": "st1" }),
    )];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    assert_eq!(alerts[0].class, "drop_at_scale");
}

#[tokio::test]
async fn drop_happy_path_warn_during_transit() {
    let det = DropAtScaleDetector::new(DropDetectorConfig::default());
    let signals = vec![sig(
        "s2",
        "dev2",
        "drop",
        json!({ "accel_g": 4.0, "custody_state": "in_transit" }),
    )];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, AlertSeverity::Warn);
}

#[tokio::test]
async fn drop_negative_below_threshold() {
    let det = DropAtScaleDetector::new(DropDetectorConfig::default());
    let signals = vec![sig(
        "s3",
        "dev3",
        "drop",
        json!({ "accel_g": 2.5, "custody_state": "station_open_voting_active" }),
    )];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

// ========== GeoFenceBreachDetector ==========

fn make_geo_det() -> GeoFenceBreachDetector {
    let fence = StationFence {
        station_id: "st1".into(),
        exterior: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
    };
    GeoFenceBreachDetector::new(GeoFenceConfig { breach_threshold_secs: 30, ..Default::default() }, vec![fence])
}

#[tokio::test]
async fn geo_fence_happy_path_breach_detected() {
    let det = make_geo_det();
    // Two signals outside fence spaced > 30s apart, voting active.
    let signals = vec![
        sig_at("g1", "dev1", "location", json!({ "lon": 5.0, "lat": 5.0, "assigned_station_id": "st1", "voting_active": true }), 0),
        sig_at("g2", "dev1", "location", json!({ "lon": 5.0, "lat": 5.0, "assigned_station_id": "st1", "voting_active": true }), 35),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(!alerts.is_empty());
    assert_eq!(alerts[0].severity, AlertSeverity::Critical);
}

#[tokio::test]
async fn geo_fence_negative_inside_fence() {
    let det = make_geo_det();
    let signals = vec![sig(
        "g3",
        "dev1",
        "location",
        json!({ "lon": 0.5, "lat": 0.5, "assigned_station_id": "st1", "voting_active": true }),
    )];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn geo_fence_below_duration_threshold() {
    let det = make_geo_det();
    // Only 10 seconds outside - below 30s threshold.
    let signals = vec![
        sig_at("g4", "dev1", "location", json!({ "lon": 5.0, "lat": 5.0, "assigned_station_id": "st1", "voting_active": true }), 0),
        sig_at("g5", "dev1", "location", json!({ "lon": 5.0, "lat": 5.0, "assigned_station_id": "st1", "voting_active": true }), 10),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

// ========== SessionHijackDetector ==========

#[tokio::test]
async fn session_hijack_happy_path_fires_critical() {
    let det = SessionHijackDetector::new(SessionHijackConfig::default());
    let signals = vec![
        sig("h1", "dev1", "biometric_score", json!({ "session_id": "sess1", "officer_id": "off1", "mahalanobis_distance": 4.0 })),
        sig("h2", "dev1", "location", json!({ "session_id": "sess1", "changed": true })),
        sig("h3", "dev1", "attestation", json!({ "session_id": "sess1", "passed": false })),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(!alerts.is_empty());
    assert_eq!(alerts[0].severity, AlertSeverity::Critical);
}

#[tokio::test]
async fn session_hijack_negative_low_score() {
    let det = SessionHijackDetector::new(SessionHijackConfig::default());
    let signals = vec![sig(
        "h4",
        "dev1",
        "biometric_score",
        json!({ "session_id": "sess2", "officer_id": "off1", "mahalanobis_distance": 1.0 }),
    )];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn session_hijack_threshold_boundary() {
    let config = SessionHijackConfig { score_threshold: 3.0, ..Default::default() };
    let det = SessionHijackDetector::new(config);
    // Score = md(3.0) + location(2.0) = 5.0 > threshold 3.0
    let signals = vec![
        sig("h5", "dev1", "biometric_score", json!({ "session_id": "sess3", "officer_id": "off1", "mahalanobis_distance": 3.0 })),
        sig("h6", "dev1", "location", json!({ "session_id": "sess3", "changed": true })),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(!alerts.is_empty());
}

// ========== BatteryStarvationClusterDetector ==========

#[tokio::test]
async fn battery_cluster_happy_path_fires() {
    let det = BatteryStarvationClusterDetector::new(BatteryStarvationConfig {
        cluster_min_devices: 2,
        ..Default::default()
    });
    let signals = vec![
        sig("b1", "dev1", "battery", json!({ "level_pct": 10.0, "lon": 1.0, "lat": 1.0 })),
        sig("b2", "dev2", "battery", json!({ "level_pct": 15.0, "lon": 1.1, "lat": 1.1 })),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(!alerts.is_empty());
    assert_eq!(alerts[0].class, "battery_starvation_cluster");
}

#[tokio::test]
async fn battery_cluster_negative_sufficient_battery() {
    let det = BatteryStarvationClusterDetector::new(BatteryStarvationConfig::default());
    let signals = vec![sig(
        "b3",
        "dev1",
        "battery",
        json!({ "level_pct": 80.0, "lon": 1.0, "lat": 1.0 }),
    )];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn battery_cluster_below_min_devices() {
    let det = BatteryStarvationClusterDetector::new(BatteryStarvationConfig {
        cluster_min_devices: 3,
        ..Default::default()
    });
    let signals = vec![
        sig("b4", "dev1", "battery", json!({ "level_pct": 5.0, "lon": 1.0, "lat": 1.0 })),
        sig("b5", "dev2", "battery", json!({ "level_pct": 5.0, "lon": 1.1, "lat": 1.1 })),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

// ========== RfidAbsenceAnomalyDetector ==========

#[tokio::test]
async fn rfid_absence_happy_path_fires() {
    let mut expected = HashMap::new();
    expected.insert("dev1".to_string(), {
        let mut s = HashSet::new();
        s.insert("BOX-001".to_string());
        s
    });
    let det = RfidAbsenceAnomalyDetector::new(
        RfidAbsenceConfig { absence_threshold_minutes: 1, ..Default::default() },
        expected,
    );
    // Custody transition happened 2 minutes ago; no BOX-001 scan.
    let signals = vec![
        sig_at("r1", "dev1", "custody_transition", json!({ "state": "in_transit" }), -120),
        sig_at("r2", "dev1", "rfid_scan", json!({ "rfid_tag": "OTHER-TAG" }), -60),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(!alerts.is_empty());
    assert_eq!(alerts[0].class, "rfid_absence_anomaly");
}

#[tokio::test]
async fn rfid_absence_negative_tag_seen() {
    let mut expected = HashMap::new();
    expected.insert("dev1".to_string(), {
        let mut s = HashSet::new();
        s.insert("BOX-001".to_string());
        s
    });
    let det = RfidAbsenceAnomalyDetector::new(RfidAbsenceConfig::default(), expected);
    let signals = vec![
        sig_at("r3", "dev1", "custody_transition", json!({ "state": "in_transit" }), -1000),
        sig_at("r4", "dev1", "rfid_scan", json!({ "rfid_tag": "BOX-001" }), -500),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn rfid_absence_threshold_boundary() {
    let mut expected = HashMap::new();
    expected.insert("dev1".to_string(), {
        let mut s = HashSet::new();
        s.insert("BOX-002".to_string());
        s
    });
    // Threshold = 16 minutes; elapsed = 15 min -> no alert.
    let det = RfidAbsenceAnomalyDetector::new(
        RfidAbsenceConfig { absence_threshold_minutes: 16, ..Default::default() },
        expected,
    );
    let signals = vec![
        sig_at("r5", "dev1", "custody_transition", json!({ "state": "in_transit" }), -900),
        sig_at("r6", "dev1", "rfid_scan", json!({ "rfid_tag": "WRONG" }), 0),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

// ========== LoginTimeAnomalyDetector ==========

fn make_login_det(threshold: f64) -> LoginTimeAnomalyDetector {
    let mut baselines = HashMap::new();
    baselines.insert(
        ("off1".to_string(), "st1".to_string()),
        LoginBaseline { mean_hour: 9.0, std_hour: 1.0 },
    );
    LoginTimeAnomalyDetector::new(
        LoginTimeConfig { zscore_threshold: threshold, ..Default::default() },
        baselines,
    )
}

#[tokio::test]
async fn login_time_happy_path_anomalous_hour() {
    let det = make_login_det(2.5);
    let mut sig1 = sig(
        "l1",
        "dev1",
        "login",
        json!({ "officer_id": "off1", "station_id": "st1" }),
    );
    // Force received_at to 02:00 (hour 2, which is 7 std devs from mean 9).
    sig1.received_at = chrono::DateTime::parse_from_rfc3339("2026-08-21T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let alerts = det.evaluate(&vec![sig1]).await.unwrap();
    assert!(!alerts.is_empty());
    assert_eq!(alerts[0].severity, AlertSeverity::Warn);
}

#[tokio::test]
async fn login_time_negative_normal_hour() {
    let det = make_login_det(2.5);
    let mut sig1 = sig(
        "l2",
        "dev1",
        "login",
        json!({ "officer_id": "off1", "station_id": "st1" }),
    );
    sig1.received_at = chrono::DateTime::parse_from_rfc3339("2026-08-21T09:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let alerts = det.evaluate(&vec![sig1]).await.unwrap();
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn login_time_threshold_boundary_exact() {
    // Z-score = (9 - 9) / 1 = 0, no alert even with low threshold.
    let det = make_login_det(0.1);
    let mut sig1 = sig(
        "l3",
        "dev1",
        "login",
        json!({ "officer_id": "off1", "station_id": "st1" }),
    );
    sig1.received_at = chrono::DateTime::parse_from_rfc3339("2026-08-21T09:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let alerts = det.evaluate(&vec![sig1]).await.unwrap();
    assert!(alerts.is_empty());
}

// ========== ReadHeavyAnomalyDetector ==========

#[tokio::test]
async fn read_heavy_happy_path_fires() {
    let det = ReadHeavyAnomalyDetector::new(ReadHeavyConfig {
        min_read_count: 5,
        read_write_ratio_threshold: 3.0,
        ..Default::default()
    });
    let mut signals = Vec::new();
    for i in 0..6 {
        signals.push(sig(
            &format!("rh{i}"),
            "dev1",
            "data_access",
            json!({ "officer_id": "off1", "session_id": "sess1", "operation": "read" }),
        ));
    }
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(!alerts.is_empty());
    assert_eq!(alerts[0].class, "read_heavy_anomaly");
}

#[tokio::test]
async fn read_heavy_negative_writes_balance() {
    let det = ReadHeavyAnomalyDetector::new(ReadHeavyConfig {
        min_read_count: 5,
        read_write_ratio_threshold: 3.0,
        ..Default::default()
    });
    let mut signals = Vec::new();
    for i in 0..6 {
        signals.push(sig(
            &format!("rh-r{i}"),
            "dev1",
            "data_access",
            json!({ "officer_id": "off1", "session_id": "sess1", "operation": "read" }),
        ));
        signals.push(sig(
            &format!("rh-w{i}"),
            "dev1",
            "data_access",
            json!({ "officer_id": "off1", "session_id": "sess1", "operation": "write" }),
        ));
    }
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn read_heavy_below_count_threshold() {
    let det = ReadHeavyAnomalyDetector::new(ReadHeavyConfig {
        min_read_count: 100,
        read_write_ratio_threshold: 3.0,
        ..Default::default()
    });
    let signals = vec![
        sig("rh0", "dev1", "data_access", json!({ "officer_id": "off1", "session_id": "sess1", "operation": "read" })),
        sig("rh1", "dev1", "data_access", json!({ "officer_id": "off1", "session_id": "sess1", "operation": "read" })),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

// ========== UebaDetector ==========

#[tokio::test]
async fn ueba_happy_path_anomaly_fires() {
    let det = UebaDetector::new(UebaConfig::default());
    let signals = vec![sig(
        "u1",
        "dev1",
        "ueba_score",
        json!({ "entity_id": "off1", "entity_type": "officer", "anomaly_score": 0.9 }),
    )];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(!alerts.is_empty());
    assert_eq!(alerts[0].class, "ueba_anomaly");
}

#[tokio::test]
async fn ueba_negative_score_below_threshold() {
    let det = UebaDetector::new(UebaConfig::default());
    let signals = vec![sig(
        "u2",
        "dev1",
        "ueba_score",
        json!({ "entity_id": "off1", "entity_type": "officer", "anomaly_score": 0.5 }),
    )];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn ueba_threshold_boundary_exact() {
    let det = UebaDetector::new(UebaConfig { score_threshold: 0.75, ..Default::default() });
    // Exactly at threshold: should NOT fire (strictly greater than).
    let signals = vec![sig(
        "u3",
        "dev1",
        "ueba_score",
        json!({ "entity_id": "off1", "entity_type": "officer", "anomaly_score": 0.75 }),
    )];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

// ========== CoordinatedDeviceAnomalyDetector ==========

#[tokio::test]
async fn coordinated_happy_path_fires_critical() {
    let det = CoordinatedDeviceAnomalyDetector::new(CoordinatedConfig {
        min_devices: 2,
        ..Default::default()
    });
    let signals = vec![
        sig("c1", "dev1", "anomaly_event", json!({ "anomaly_class": "drop" })),
        sig("c2", "dev2", "anomaly_event", json!({ "anomaly_class": "drop" })),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(!alerts.is_empty());
    assert_eq!(alerts[0].severity, AlertSeverity::Critical);
}

#[tokio::test]
async fn coordinated_negative_single_device() {
    let det = CoordinatedDeviceAnomalyDetector::new(CoordinatedConfig {
        min_devices: 3,
        ..Default::default()
    });
    let signals = vec![
        sig("c3", "dev1", "anomaly_event", json!({ "anomaly_class": "drop" })),
        sig("c4", "dev2", "anomaly_event", json!({ "anomaly_class": "drop" })),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn coordinated_threshold_boundary_class_not_watched() {
    let det = CoordinatedDeviceAnomalyDetector::new(CoordinatedConfig {
        min_devices: 2,
        ..Default::default()
    });
    let signals = vec![
        sig("c5", "dev1", "anomaly_event", json!({ "anomaly_class": "unknown_class" })),
        sig("c6", "dev2", "anomaly_event", json!({ "anomaly_class": "unknown_class" })),
        sig("c7", "dev3", "anomaly_event", json!({ "anomaly_class": "unknown_class" })),
    ];
    let alerts = det.evaluate(&signals).await.unwrap();
    assert!(alerts.is_empty());
}

// ========== Pipeline ==========

#[tokio::test]
async fn pipeline_fanout_to_all_enabled_detectors() {
    use eel_ai_platform_telemetry_analytics::{
        dispatch::{AlertDispatcher, DispatchConfig},
        pipeline::AnalyticsPipeline,
    };
    let det1: Arc<dyn Detector> = Arc::new(DropAtScaleDetector::new(DropDetectorConfig::default()));
    let det2: Arc<dyn Detector> = Arc::new(UebaDetector::new(UebaConfig::default()));
    let dispatcher = Arc::new(AlertDispatcher::new(DispatchConfig {
        ecc_webhook_url: "http://127.0.0.1:19999/no-such".into(),
        max_retries: 0,
        circuit_open_threshold: 10,
    }));
    let pipeline = AnalyticsPipeline::new(vec![det1, det2], dispatcher);

    let signals = vec![
        sig("p1", "dev1", "drop", json!({ "accel_g": 5.0, "custody_state": "station_open_voting_active" })),
        sig("p2", "dev1", "ueba_score", json!({ "entity_id": "off1", "entity_type": "officer", "anomaly_score": 0.9 })),
    ];
    // Both detectors should fire.
    let alerts = pipeline.process(signals).await;
    assert_eq!(alerts.len(), 2);
}

#[tokio::test]
async fn pipeline_skips_disabled_detector() {
    use eel_ai_platform_telemetry_analytics::{
        dispatch::{AlertDispatcher, DispatchConfig},
        pipeline::AnalyticsPipeline,
    };
    let mut config = DropDetectorConfig::default();
    config.enabled = false;
    let det: Arc<dyn Detector> = Arc::new(DropAtScaleDetector::new(config));
    let dispatcher = Arc::new(AlertDispatcher::new(DispatchConfig {
        ecc_webhook_url: "http://127.0.0.1:19999/no-such".into(),
        max_retries: 0,
        circuit_open_threshold: 10,
    }));
    let pipeline = AnalyticsPipeline::new(vec![det], dispatcher);

    let signals = vec![sig(
        "p3",
        "dev1",
        "drop",
        json!({ "accel_g": 5.0, "custody_state": "station_open_voting_active" }),
    )];
    let alerts = pipeline.process(signals).await;
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn pipeline_dispatch_failure_does_not_panic() {
    use eel_ai_platform_telemetry_analytics::{
        dispatch::{AlertDispatcher, DispatchConfig},
        pipeline::AnalyticsPipeline,
    };
    let det: Arc<dyn Detector> = Arc::new(DropAtScaleDetector::new(DropDetectorConfig::default()));
    // Point to a port that is definitely not listening.
    let dispatcher = Arc::new(AlertDispatcher::new(DispatchConfig {
        ecc_webhook_url: "http://127.0.0.1:19998/ecc".into(),
        max_retries: 0,
        circuit_open_threshold: 10,
    }));
    let pipeline = AnalyticsPipeline::new(vec![det], dispatcher);

    let signals = vec![sig(
        "p4",
        "dev1",
        "drop",
        json!({ "accel_g": 5.0, "custody_state": "station_open_voting_active" }),
    )];
    // Should not panic even when dispatch fails.
    let alerts = pipeline.process(signals).await;
    assert!(!alerts.is_empty());
}

// ========== Rule Registry ==========

#[tokio::test]
async fn rule_registry_load_from_yaml() {
    let path = rules_yaml_path();
    let registry = RuleRegistry::load(&path).expect("registry load");
    assert!(!registry.all().is_empty());
}

#[tokio::test]
async fn rule_registry_hot_reload_upsert() {
    let path = rules_yaml_path();
    let registry = RuleRegistry::load(&path).expect("registry load");
    let new_rule = RuleEntry {
        id: "test_rule".into(),
        class: "TestDetector".into(),
        severity_threshold: 1.0,
        enabled: true,
        params: Default::default(),
    };
    registry.upsert(new_rule).unwrap();
    assert!(registry.get("test_rule").is_some());
}

#[tokio::test]
async fn rule_registry_missing_field_validation() {
    let bad_yaml = "rules:\n  - id: \"\"\n    class: Foo\n    severity_threshold: 1.0\n    enabled: true\n";
    let path = "/private/tmp/bad-rules.yaml";
    std::fs::write(path, bad_yaml).unwrap();
    let result = RuleRegistry::load(path);
    assert!(result.is_err());
    std::fs::remove_file(path).ok();
}

// ========== Backtest ==========

#[tokio::test]
async fn backtest_perfect_precision_recall() {
    let det: Arc<dyn Detector> = Arc::new(DropAtScaleDetector::new(DropDetectorConfig::default()));
    let now = Utc::now();
    let signals = vec![sig(
        "bt1",
        "dev1",
        "drop",
        json!({ "accel_g": 5.0, "custody_state": "station_open_voting_active" }),
    )];
    let labels = vec![LabelledAnomaly {
        device_id: "dev1".into(),
        class: "drop_at_scale".into(),
        start: now - chrono::Duration::seconds(60),
        end: now + chrono::Duration::seconds(60),
    }];
    let metrics = run_backtest(det, signals, &labels).await;
    assert_eq!(metrics.true_positive, 1);
    assert_eq!(metrics.false_positive, 0);
    assert!((metrics.precision - 1.0).abs() < 1e-9);
    assert!((metrics.recall - 1.0).abs() < 1e-9);
}

#[tokio::test]
async fn backtest_zero_precision_all_fp() {
    let det: Arc<dyn Detector> = Arc::new(DropAtScaleDetector::new(DropDetectorConfig::default()));
    let signals = vec![sig(
        "bt2",
        "dev1",
        "drop",
        json!({ "accel_g": 5.0, "custody_state": "in_transit" }),
    )];
    // Label for a different device - alert becomes FP.
    let now = Utc::now();
    let labels = vec![LabelledAnomaly {
        device_id: "dev_other".into(),
        class: "drop_at_scale".into(),
        start: now - chrono::Duration::seconds(60),
        end: now + chrono::Duration::seconds(60),
    }];
    let metrics = run_backtest(det, signals, &labels).await;
    assert_eq!(metrics.false_positive, 1);
    assert!((metrics.precision).abs() < 1e-9);
}

// ========== Alert Dispatcher ==========

fn make_dispatcher(url: &str, retries: u32) -> AlertDispatcher {
    AlertDispatcher::new(DispatchConfig {
        ecc_webhook_url: url.to_string(),
        max_retries: retries,
        circuit_open_threshold: 5,
    })
}

fn make_alert() -> Alert {
    Alert::new("test_rule", "drop_at_scale", AlertSeverity::Critical, json!({}))
}

#[tokio::test]
async fn dispatcher_circuit_breaker_opens_after_failures() {
    let dispatcher = AlertDispatcher::new(DispatchConfig {
        ecc_webhook_url: "http://127.0.0.1:19997/ecc".into(),
        max_retries: 0,
        circuit_open_threshold: 2,
    });
    let alert = make_alert();
    // Exhaust failure threshold.
    let _ = dispatcher.dispatch(&alert).await;
    let _ = dispatcher.dispatch(&alert).await;
    // Circuit should now be open.
    let result = dispatcher.dispatch(&alert).await;
    assert!(matches!(
        result,
        Err(eel_ai_platform_telemetry_analytics::dispatch::DispatchError::CircuitOpen)
    ));
}

#[tokio::test]
async fn dispatcher_cloud_event_structure() {
    let dispatcher = make_dispatcher("http://127.0.0.1:19996/ecc", 0);
    let alert = make_alert();
    let event = dispatcher.build_cloud_event(&alert);
    assert_eq!(event["specversion"], "1.0");
    assert_eq!(event["type"], "eel.telemetry.alert.v1");
    assert_eq!(event["id"], alert.id);
}

#[tokio::test]
async fn dispatcher_alert_id_is_unique_per_alert() {
    let a1 = make_alert();
    let a2 = make_alert();
    // UUID v7 per alert: ids must differ.
    assert_ne!(a1.id, a2.id);
}

// ========== API Routes ==========

#[tokio::test]
async fn api_rules_crud_roundtrip() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use eel_ai_platform_telemetry_analytics::api::{router, ApiState};
    use tower::ServiceExt;

    let path = rules_yaml_path();
    let registry = Arc::new(RuleRegistry::load(&path).expect("load"));
    let state = ApiState { registry };
    let app = router(state);

    // List rules.
    let req = Request::builder()
        .uri("/v1/analytics/rules")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Create a rule.
    let body = serde_json::to_string(&RuleEntry {
        id: "api_test_rule".into(),
        class: "TestDetector".into(),
        severity_threshold: 1.0,
        enabled: true,
        params: Default::default(),
    })
    .unwrap();
    let req = Request::builder()
        .uri("/v1/analytics/rules")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn api_replay_refuses_rather_than_being_accepted_and_dropped() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use eel_ai_platform_telemetry_analytics::api::{router, ApiState};
    use tower::ServiceExt;

    let path = rules_yaml_path();
    let registry = Arc::new(RuleRegistry::load(&path).expect("load"));
    let state = ApiState { registry };
    let app = router(state);

    let body = serde_json::json!({
        "detector_id": "drop_at_scale",
        "window_start": "2026-08-01T00:00:00Z",
        "window_end": "2026-08-02T00:00:00Z",
    });
    let req = Request::builder()
        .uri("/v1/analytics/replay")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    // Previously 202 Accepted: the handler logged the request and returned, so a
    // caller held a receipt for a replay that was never run.
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
}
