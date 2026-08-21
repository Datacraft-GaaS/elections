use chrono::Utc;
use eel_abis_modality_signature::{
    matcher::{dtw, match_signature, DEFAULT_THRESHOLD},
    parser::{parse_signature, serialize_signature, SignatureError},
    quality::score_quality,
    SignatureSample, SignatureTemplate,
};

fn make_template(
    n: usize,
    sample_rate_hz: u16,
    pressure_val: f32,
    velocity_val: f32,
    accel_val: f32,
    strokes: u16,
    pen_up_count: u16,
) -> SignatureTemplate {
    let samples: Vec<SignatureSample> = (0..n)
        .map(|i| SignatureSample {
            timestamp_ms: i as u32 * (1000 / sample_rate_hz.max(1) as u32),
            pressure: pressure_val,
            velocity: velocity_val,
            acceleration: accel_val,
            pen_up: false,
        })
        .collect();

    let total_duration_ms = samples.last().map(|s| s.timestamp_ms).unwrap_or(0);

    SignatureTemplate {
        template_id: "test".to_string(),
        capture_device: None,
        samples,
        pen_up_count,
        sample_rate_hz,
        total_duration_ms,
        stroke_count: strokes,
        captured_at: Utc::now(),
    }
}

fn make_template_with_pen_ups(
    n: usize,
    sample_rate_hz: u16,
    pressure_val: f32,
) -> SignatureTemplate {
    // Create a template where every 10th sample is pen_up.
    let samples: Vec<SignatureSample> = (0..n)
        .map(|i| SignatureSample {
            timestamp_ms: i as u32 * 10,
            pressure: pressure_val,
            velocity: 0.5,
            acceleration: 0.0,
            pen_up: i % 10 == 9,
        })
        .collect();

    let pen_up_count = samples.iter().filter(|s| s.pen_up).count() as u16;
    let stroke_count = (n / 10) as u16;
    let total_duration_ms = samples.last().map(|s| s.timestamp_ms).unwrap_or(0);

    SignatureTemplate {
        template_id: "test-pu".to_string(),
        capture_device: None,
        samples,
        pen_up_count,
        sample_rate_hz,
        total_duration_ms,
        stroke_count,
        captured_at: Utc::now(),
    }
}

// Test 1: parse round-trip
#[test]
fn test_parse_round_trip() {
    let tmpl = make_template_with_pen_ups(30, 100, 0.5);
    let bytes = serialize_signature(&tmpl);
    let parsed = parse_signature(&bytes).expect("should parse");
    assert_eq!(parsed.samples.len(), tmpl.samples.len());
    assert_eq!(parsed.sample_rate_hz, tmpl.sample_rate_hz);
    for (p, t) in parsed.samples.iter().zip(tmpl.samples.iter()) {
        assert!((p.pressure - t.pressure).abs() < 1e-5);
        assert!((p.velocity - t.velocity).abs() < 1e-5);
        assert!((p.acceleration - t.acceleration).abs() < 1e-5);
        assert_eq!(p.pen_up, t.pen_up);
    }
}

// Test 2: parse rejects wrong magic
#[test]
fn test_parse_wrong_magic() {
    let tmpl = make_template(4, 100, 0.5, 0.0, 0.0, 1, 0);
    let mut bytes = serialize_signature(&tmpl);
    bytes[0] = 0xFF;
    assert!(matches!(
        parse_signature(&bytes),
        Err(SignatureError::InvalidMagic)
    ));
}

// Test 3: parse rejects wrong version
#[test]
fn test_parse_wrong_version() {
    let tmpl = make_template(4, 100, 0.5, 0.0, 0.0, 1, 0);
    let mut bytes = serialize_signature(&tmpl);
    bytes[3] = 99;
    assert!(matches!(
        parse_signature(&bytes),
        Err(SignatureError::UnsupportedVersion(99))
    ));
}

// Test 4: parse empty samples
#[test]
fn test_parse_empty_samples() {
    let tmpl = make_template(0, 100, 0.5, 0.0, 0.0, 0, 0);
    let bytes = serialize_signature(&tmpl);
    let parsed = parse_signature(&bytes).expect("empty should parse");
    assert_eq!(parsed.samples.len(), 0);
}

// Test 5: dtw of identical series = 0.0
#[test]
fn test_dtw_identical() {
    let series: Vec<(f32, f32, f32)> = (0..10).map(|i| (i as f32 * 0.1, 0.5, 0.0)).collect();
    let dist = dtw(&series, &series);
    assert!(
        dist.abs() < 1e-5,
        "dtw of identical series should be 0, got {dist}"
    );
}

// Test 6: dtw of reversed series > 0.0
#[test]
fn test_dtw_reversed() {
    let series: Vec<(f32, f32, f32)> = (0..10).map(|i| (i as f32 * 0.1, 0.5, 0.0)).collect();
    let reversed: Vec<(f32, f32, f32)> = series.iter().copied().rev().collect();
    let dist = dtw(&series, &reversed);
    assert!(
        dist > 0.0,
        "reversed series dtw should exceed 0, got {dist}"
    );
}

// Test 7: match genuine pair (same template twice) > threshold
#[test]
fn test_match_genuine_pair() {
    let tmpl = make_template_with_pen_ups(50, 100, 0.7);
    let score = match_signature(&tmpl, &tmpl);
    assert!(
        score.raw >= DEFAULT_THRESHOLD,
        "genuine pair score {} should exceed threshold {}",
        score.raw,
        DEFAULT_THRESHOLD
    );
    assert!(score.threshold_pass);
}

// Test 8: match imposter pair (constant 0 vs constant 1 pressure) < threshold
#[test]
fn test_match_imposter_pair() {
    let probe = make_template(50, 100, 0.0, 0.0, 0.0, 1, 0);
    let gallery = make_template(50, 100, 1.0, 1.0, 1.0, 1, 0);
    let score = match_signature(&probe, &gallery);
    assert!(
        score.raw < DEFAULT_THRESHOLD,
        "imposter score {} should be below threshold {}",
        score.raw,
        DEFAULT_THRESHOLD
    );
}

// Test 9: quality 100Hz sample rate scores 100
#[test]
fn test_quality_100hz_scores_100() {
    let tmpl = make_template(0, 100, 0.5, 0.0, 0.0, 0, 0);
    let q = score_quality(&tmpl);
    assert!((q.sample_rate_score - 100.0).abs() < 1e-3);
}

// Test 10: quality 25Hz sample rate scores lower
#[test]
fn test_quality_25hz_scores_lower() {
    let tmpl_100 = make_template(0, 100, 0.5, 0.0, 0.0, 0, 0);
    let tmpl_25 = make_template(0, 25, 0.5, 0.0, 0.0, 0, 0);
    let q100 = score_quality(&tmpl_100);
    let q25 = score_quality(&tmpl_25);
    assert!(q25.sample_rate_score < q100.sample_rate_score);
}

// Test 11: quality 0 strokes returns 0 stroke_score
#[test]
fn test_quality_zero_strokes() {
    let tmpl = make_template(10, 100, 0.5, 0.0, 0.0, 0, 0);
    let q = score_quality(&tmpl);
    assert_eq!(q.stroke_score, 0.0);
}

// Test 12: quality normal signature returns overall in [0, 100]
#[test]
fn test_quality_overall_in_range() {
    let tmpl = make_template_with_pen_ups(100, 100, 0.7);
    let q = score_quality(&tmpl);
    assert!(
        q.overall >= 0.0 && q.overall <= 100.0,
        "overall {} out of range",
        q.overall
    );
}

// Test 13: pen-up count matches parse result
#[test]
fn test_pen_up_count_matches() {
    let original = make_template_with_pen_ups(30, 100, 0.6);
    let bytes = serialize_signature(&original);
    let parsed = parse_signature(&bytes).expect("should parse");
    assert_eq!(parsed.pen_up_count, original.pen_up_count);
}

// Test 14: stroke count accurately computed
#[test]
fn test_stroke_count_computed() {
    // make_template_with_pen_ups creates n/10 strokes.
    let tmpl = make_template_with_pen_ups(30, 100, 0.5);
    // 30 samples with pen_up at index 9, 19, 29 -> 3 strokes.
    let bytes = serialize_signature(&tmpl);
    let parsed = parse_signature(&bytes).expect("should parse");
    assert_eq!(
        parsed.stroke_count, 3,
        "expected 3 strokes, got {}",
        parsed.stroke_count
    );
}
