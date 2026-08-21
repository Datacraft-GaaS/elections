use chrono::Utc;
use eel_abis_modality_latent::{
    domain::{CustodyEvent, LatentSource, LatentTemplate, Minutia, MinutiaType},
    matcher::{match_latent, DEFAULT_THRESHOLD},
    parser::{append_custody_event, parse_nist_type13, serialize_nist_type13, LatentError},
    quality::score_quality,
};

fn make_minutia(x: u16, y: u16, angle: u8, quality: u8) -> Minutia {
    Minutia {
        x,
        y,
        angle,
        quality,
        minutia_type: MinutiaType::RidgeEnding,
    }
}

fn make_template(minutiae: Vec<Minutia>, distortion: f32, usable: f32) -> LatentTemplate {
    LatentTemplate {
        template_id: "test-l-001".into(),
        source: LatentSource::Lifted,
        lift_id: Some("lift-001".into()),
        case_ref: Some("case-2026-001".into()),
        distortion_score: distortion,
        usable_ridge_pct: usable,
        examiner_id: Some("exam-001".into()),
        chain_of_custody: vec![],
        minutiae,
        captured_at: Utc::now(),
    }
}

fn make_custody_event(action: &str) -> CustodyEvent {
    CustodyEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        actor_id: "officer-001".into(),
        action: action.into(),
        signature: None,
    }
}

fn make_nist_buf(minutiae: &[Minutia]) -> Vec<u8> {
    let t = make_template(minutiae.to_vec(), 0.0, 1.0);
    serialize_nist_type13(&t)
}

// Test 1: NIST Type-13 parse round-trip preserves minutiae count and values.
#[test]
fn test_nist_parse_round_trip() {
    let minutiae = vec![
        make_minutia(100, 200, 45, 80),
        make_minutia(300, 150, 90, 70),
    ];
    let buf = make_nist_buf(&minutiae);
    let parsed = parse_nist_type13(&buf).unwrap();
    assert_eq!(parsed.minutiae.len(), 2);
    assert_eq!(parsed.minutiae[0].x, 100);
    assert_eq!(parsed.minutiae[1].angle, 90);
}

// Test 2: parse rejects wrong magic bytes.
#[test]
fn test_parse_rejects_wrong_magic() {
    let mut buf = make_nist_buf(&[make_minutia(10, 10, 0, 50)]);
    buf[0] = 0xFF;
    let result = parse_nist_type13(&buf);
    assert!(matches!(result, Err(LatentError::InvalidMagic)));
}

// Test 3: parse rejects truncated data.
#[test]
fn test_parse_rejects_truncated() {
    let buf = vec![0x41, 0x4E, 0x13, 0x00]; // too short
    let result = parse_nist_type13(&buf);
    assert!(matches!(result, Err(LatentError::BufferTooShort { .. })));
}

// Test 4: serialize then parse produces identical minutiae count.
#[test]
fn test_serialize_parse_count_identity() {
    let minutiae: Vec<Minutia> = (0..8)
        .map(|i| make_minutia(i * 30, i * 25, i as u8 * 20, 75))
        .collect();
    let buf = make_nist_buf(&minutiae);
    let parsed = parse_nist_type13(&buf).unwrap();
    assert_eq!(parsed.minutiae.len(), minutiae.len());
}

// Test 5: chain of custody append increments len.
#[test]
fn test_custody_append_increments_len() {
    let mut t = make_template(vec![], 0.0, 1.0);
    assert_eq!(t.chain_of_custody.len(), 0);
    append_custody_event(&mut t, make_custody_event("collected"));
    assert_eq!(t.chain_of_custody.len(), 1);
    append_custody_event(&mut t, make_custody_event("examined"));
    assert_eq!(t.chain_of_custody.len(), 2);
}

// Test 6: custody events preserve order.
#[test]
fn test_custody_events_preserve_order() {
    let mut t = make_template(vec![], 0.0, 1.0);
    append_custody_event(&mut t, make_custody_event("collected"));
    append_custody_event(&mut t, make_custody_event("examined"));
    append_custody_event(&mut t, make_custody_event("stored"));
    assert_eq!(t.chain_of_custody[0].action, "collected");
    assert_eq!(t.chain_of_custody[1].action, "examined");
    assert_eq!(t.chain_of_custody[2].action, "stored");
}

// Test 7: partial print matching - 30% overlap probe vs full gallery scores > threshold.
#[test]
fn test_partial_print_match_above_threshold() {
    // Gallery has 20 minutiae; probe has 6 that all match (30%)
    let gallery_min: Vec<Minutia> = (0..20)
        .map(|i| make_minutia(i * 20, i * 15, i as u8 * 5, 80))
        .collect();
    let probe_min: Vec<Minutia> = (0..6)
        .map(|i| make_minutia(i * 20, i * 15, i as u8 * 5, 80))
        .collect();
    let gallery = make_template(gallery_min, 0.0, 1.0);
    // usable_ridge_pct=1.0 ensures quality weight does not reduce
    let probe = make_template(probe_min, 0.0, 1.0);
    let score = match_latent(&probe, &gallery);
    // All probe minutiae match gallery -> raw = 1.0 * 1.0 = 1.0
    assert!(
        score.raw > DEFAULT_THRESHOLD,
        "partial match should exceed threshold, got {}",
        score.raw
    );
}

// Test 8: quality scorer penalizes high distortion.
#[test]
fn test_quality_penalizes_high_distortion() {
    let minutiae: Vec<Minutia> = (0..20)
        .map(|i| make_minutia(i * 10, i * 10, 0, 80))
        .collect();
    let clean = make_template(minutiae.clone(), 0.0, 1.0);
    let distorted = make_template(minutiae, 0.9, 1.0);
    assert!(score_quality(&clean) > score_quality(&distorted));
}

// Test 9: quality scorer returns 0 for empty minutiae.
#[test]
fn test_quality_empty_returns_zero() {
    let t = make_template(vec![], 0.0, 1.0);
    assert_eq!(score_quality(&t), 0.0);
}

// Test 10: quality score in [0, 100].
#[test]
fn test_quality_in_range() {
    let minutiae: Vec<Minutia> = (0..15)
        .map(|i| make_minutia(i * 15, i * 12, 0, 80))
        .collect();
    let t = make_template(minutiae, 0.2, 0.8);
    let q = score_quality(&t);
    assert!(q >= 0.0 && q <= 100.0, "quality out of range: {q}");
}

// Test 11: imposter pair scores below threshold.
#[test]
fn test_imposter_below_threshold() {
    let probe_min: Vec<Minutia> = (0..10)
        .map(|i| make_minutia(i * 10, i * 10, i as u8 * 10, 80))
        .collect();
    let gallery_min: Vec<Minutia> = (0..10)
        .map(|i| make_minutia(i * 10 + 250, i * 10 + 250, (i as u8).wrapping_add(90), 80))
        .collect();
    let probe = make_template(probe_min, 0.1, 0.9);
    let gallery = make_template(gallery_min, 0.1, 0.9);
    let score = match_latent(&probe, &gallery);
    assert!(
        score.raw < DEFAULT_THRESHOLD,
        "imposter score should be below threshold, got {}",
        score.raw
    );
}

// Test 12: match on empty probe returns raw=0.
#[test]
fn test_match_empty_probe_returns_zero() {
    let gallery_min = vec![make_minutia(100, 100, 45, 80)];
    let probe = make_template(vec![], 0.0, 1.0);
    let gallery = make_template(gallery_min, 0.0, 1.0);
    let score = match_latent(&probe, &gallery);
    assert_eq!(score.raw, 0.0);
    assert!(!score.threshold_pass);
}

// Test 13: matched_minutiae count is correct.
#[test]
fn test_matched_minutiae_count() {
    let minutiae: Vec<Minutia> = (0..5)
        .map(|i| make_minutia(i * 30, i * 25, i as u8 * 10, 80))
        .collect();
    let t = make_template(minutiae.clone(), 0.0, 1.0);
    let score = match_latent(&t, &t);
    assert_eq!(score.matched_minutiae, 5);
}

// Test 14: low usable_ridge_pct reduces score below threshold.
#[test]
fn test_low_usable_ridge_reduces_score() {
    let minutiae: Vec<Minutia> = (0..10)
        .map(|i| make_minutia(i * 20, i * 15, i as u8 * 5, 80))
        .collect();
    let probe = make_template(minutiae.clone(), 0.0, 0.2); // only 20% usable
    let gallery = make_template(minutiae, 0.0, 1.0);
    let score = match_latent(&probe, &gallery);
    // raw = 1.0 * 0.2 = 0.2, below DEFAULT_THRESHOLD=0.35
    assert!(
        !score.threshold_pass,
        "low usable ridge should fail threshold, raw={}",
        score.raw
    );
}
