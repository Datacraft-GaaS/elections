use eel_rts_behavioural_biometrics::{
    hijack_detector::HijackDetector,
    keystroke::{KeyEvent, KeystrokeExtractor},
    mouse::{MouseEvent, MouseExtractor},
    session::{CombinedFeatures, SessionProfile},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_key_event(dwell: f64, flight: f64, pressure: f64) -> KeyEvent {
    KeyEvent {
        key_code: 65,
        dwell_ms: dwell,
        flight_ms: flight,
        pressure,
        timestamp_ms: 1_000,
    }
}

fn make_mouse_event(x: f64, y: f64, ts: u64) -> MouseEvent {
    MouseEvent {
        x,
        y,
        timestamp_ms: ts,
        button_state: 0,
    }
}

fn make_combined(mean_dwell: f64, mean_flight: f64, mean_velocity: f64) -> CombinedFeatures {
    CombinedFeatures {
        keystroke: eel_rts_behavioural_biometrics::keystroke::KeystrokeFeatures {
            mean_dwell,
            std_dwell: 1.0,
            mean_flight,
            std_flight: 1.0,
            pressure_histogram: [0.1; 10],
        },
        mouse: eel_rts_behavioural_biometrics::mouse::MouseFeatures {
            mean_velocity,
            std_velocity: 0.5,
            mean_acceleration: 0.1,
            mean_jerk: 0.01,
            mean_curvature: 0.001,
        },
        extracted_at: "2026-08-22T00:00:00Z".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Keystroke tests
// ---------------------------------------------------------------------------

#[test]
fn keystroke_features_require_min_events() {
    let mut ext = KeystrokeExtractor::new(50);
    for _ in 0..4 {
        ext.push(make_key_event(100.0, 50.0, 0.5));
    }
    assert!(ext.features().is_none());
}

#[test]
fn keystroke_features_with_5_events() {
    let mut ext = KeystrokeExtractor::new(50);
    for _ in 0..5 {
        ext.push(make_key_event(100.0, 50.0, 0.5));
    }
    assert!(ext.features().is_some());
}

#[test]
fn keystroke_mean_computed_correctly() {
    let mut ext = KeystrokeExtractor::new(50);
    let dwells = [80.0, 100.0, 120.0, 140.0, 160.0];
    for &d in &dwells {
        ext.push(make_key_event(d, 50.0, 0.5));
    }
    let f = ext.features().unwrap();
    let expected_mean = dwells.iter().sum::<f64>() / dwells.len() as f64;
    assert!((f.mean_dwell - expected_mean).abs() < 1e-9);
}

#[test]
fn keystroke_std_computed_correctly() {
    let mut ext = KeystrokeExtractor::new(50);
    // All equal: std should be 0.
    for _ in 0..5 {
        ext.push(make_key_event(100.0, 50.0, 0.5));
    }
    let f = ext.features().unwrap();
    assert!(f.std_dwell.abs() < 1e-9);
}

#[test]
fn keystroke_pressure_histogram_normalised() {
    let mut ext = KeystrokeExtractor::new(50);
    for i in 0..10 {
        ext.push(make_key_event(100.0, 50.0, i as f64 * 0.1));
    }
    let f = ext.features().unwrap();
    let sum: f64 = f.pressure_histogram.iter().sum();
    assert!((sum - 1.0).abs() < 1e-9, "histogram sum was {sum}");
}

// ---------------------------------------------------------------------------
// Mouse tests
// ---------------------------------------------------------------------------

#[test]
fn mouse_features_require_min_events() {
    let mut ext = MouseExtractor::new(50);
    ext.push(make_mouse_event(0.0, 0.0, 0));
    ext.push(make_mouse_event(10.0, 10.0, 100));
    assert!(ext.features().is_none());
}

#[test]
fn mouse_velocity_computed() {
    let mut ext = MouseExtractor::new(50);
    ext.push(make_mouse_event(0.0, 0.0, 0));
    ext.push(make_mouse_event(100.0, 0.0, 100));
    ext.push(make_mouse_event(200.0, 0.0, 200));
    let f = ext.features().unwrap();
    assert!(f.mean_velocity > 0.0, "velocity should be positive");
}

#[test]
fn mouse_acceleration_computed() {
    let mut ext = MouseExtractor::new(50);
    // Constant velocity: acceleration should be 0.
    for i in 0..5 {
        ext.push(make_mouse_event(i as f64 * 100.0, 0.0, i as u64 * 100));
    }
    let f = ext.features().unwrap();
    // Mean acceleration is close to 0 for constant velocity.
    assert!(
        f.mean_acceleration.abs() < 1.0,
        "acceleration: {}",
        f.mean_acceleration
    );
}

// ---------------------------------------------------------------------------
// Session tests
// ---------------------------------------------------------------------------

#[test]
fn session_learning_phase_returns_none() {
    let mut profile = SessionProfile::new("officer-1".into(), "sess-1".into());
    for i in 0..10 {
        let result = profile.ingest(make_combined(100.0 + i as f64, 50.0, 1.0));
        assert!(result.is_none(), "should be None during learning phase");
    }
}

#[test]
fn session_baseline_established_after_window() {
    let mut profile = SessionProfile::new("officer-1".into(), "sess-2".into());
    // Ingest enough samples to exit the learning phase.
    for i in 0..profile.baseline_window {
        profile.ingest(make_combined(100.0 + (i % 10) as f64, 50.0, 1.0));
    }
    assert!(profile.baseline.is_some(), "baseline should be established");
    // One more sample should return a distance.
    let result = profile.ingest(make_combined(100.0, 50.0, 1.0));
    assert!(result.is_some());
}

#[test]
fn session_mahalanobis_zero_for_identical() {
    let mut profile = SessionProfile::new("officer-1".into(), "sess-3".into());
    // Build baseline with constant features.
    for _ in 0..profile.baseline_window {
        profile.ingest(make_combined(100.0, 50.0, 1.0));
    }
    // Inject the exact same feature vector.
    let dist = profile.ingest(make_combined(100.0, 50.0, 1.0)).unwrap();
    assert!(
        dist < 1e-3,
        "distance for identical features should be near 0, got {dist}"
    );
}

#[test]
fn session_different_user_distinct_baseline() {
    let bw = {
        let p = SessionProfile::new("x".into(), "s".into());
        p.baseline_window
    };

    let mut profile_a = SessionProfile::new("officer-a".into(), "sess-a".into());
    let mut profile_b = SessionProfile::new("officer-b".into(), "sess-b".into());

    for i in 0..bw {
        profile_a.ingest(make_combined(80.0 + (i % 5) as f64, 40.0, 0.5));
        profile_b.ingest(make_combined(200.0 + (i % 5) as f64, 100.0, 2.0));
    }

    let mean_a = profile_a.baseline.as_ref().unwrap().mean_vec[0];
    let mean_b = profile_b.baseline.as_ref().unwrap().mean_vec[0];
    assert!(
        (mean_a - mean_b).abs() > 50.0,
        "baselines should differ: {mean_a} vs {mean_b}"
    );
}

// ---------------------------------------------------------------------------
// HijackDetector tests
// ---------------------------------------------------------------------------

#[test]
fn hijack_below_threshold_no_event() {
    let detector = HijackDetector::new(5.0);
    let profile = SessionProfile::new("officer-1".into(), "sess-1".into());
    let result = detector.check(&profile, 3.0);
    assert!(result.is_none());
}

#[test]
fn hijack_above_threshold_emits_event() {
    let detector = HijackDetector::new(5.0);
    let profile = SessionProfile::new("officer-1".into(), "sess-1".into());
    let result = detector.check(&profile, 6.0);
    assert!(result.is_some());
}

#[test]
fn hijack_event_correct_type_field() {
    let detector = HijackDetector::new(5.0);
    let profile = SessionProfile::new("officer-1".into(), "sess-1".into());
    let event = detector.check(&profile, 10.0).unwrap();
    assert_eq!(event.event_type, "eel.rts.session_hijack_suspected.v1");
}

// ---------------------------------------------------------------------------
// Stability test
// ---------------------------------------------------------------------------

#[test]
fn session_100_synthetic_events_stable_baseline() {
    let mut profile = SessionProfile::new("officer-stable".into(), "sess-stable".into());
    let bw = profile.baseline_window;

    // Feed baseline_window samples to establish the baseline.
    for i in 0..bw {
        profile.ingest(make_combined(100.0 + (i % 3) as f64, 50.0, 1.0));
    }
    assert!(profile.baseline.is_some());

    // Feed 100 more samples from the same distribution; distances should remain modest.
    let mut max_dist = 0.0f64;
    for i in 0..100 {
        if let Some(d) = profile.ingest(make_combined(100.0 + (i % 3) as f64, 50.0, 1.0)) {
            if d > max_dist {
                max_dist = d;
            }
        }
    }
    // All distances should be well below a suspicious threshold.
    assert!(
        max_dist < 20.0,
        "max distance from stable distribution was {max_dist}"
    );
}
