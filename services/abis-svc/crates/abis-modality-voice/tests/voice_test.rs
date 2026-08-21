use chrono::Utc;
use eel_abis_modality_voice::{
    domain::{UtteranceType, VoiceTemplate},
    matcher::{
        cosine_similarity, dtw_distance, match_voice_cosine, match_voice_dtw,
        DEFAULT_THRESHOLD_COSINE, DEFAULT_THRESHOLD_DTW,
    },
    parser::{
        deserialize_embedding, extract_mfcc, fft_inplace, parse_wav, parse_wav_header,
        serialize_embedding, VoiceError,
    },
    quality::score_quality,
};

// Build a minimal synthetic WAV buffer.
fn make_wav_buf(num_samples: usize, sample_rate: u32) -> Vec<u8> {
    let data_size = num_samples * 2; // 16-bit mono
    let total_size = 36 + data_size;
    let mut buf = Vec::with_capacity(total_size + 8);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(total_size as u32).to_le_bytes()); // file size - 8
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buf.extend_from_slice(&1u16.to_le_bytes()); // channels
    buf.extend_from_slice(&sample_rate.to_le_bytes()); // sample rate
    let byte_rate = sample_rate * 2;
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&(data_size as u32).to_le_bytes());

    // PCM samples (silent)
    buf.extend(std::iter::repeat(0u8).take(data_size));
    buf
}

fn make_template_with_embedding(embedding: Vec<f32>, mfcc: Vec<Vec<f32>>) -> VoiceTemplate {
    VoiceTemplate {
        template_id: "voice-test".into(),
        utterance_type: UtteranceType::FreeSpeech,
        sample_rate_hz: 16000,
        duration_ms: 3000,
        language: "en".into(),
        embedding_dim: embedding.len() as u16,
        embedding,
        mfcc_features: mfcc,
        snr_db: Some(30.0),
        captured_at: Utc::now(),
    }
}

// Test 1: WAV header parse with synthetic 44-byte WAV header.
#[test]
fn test_wav_header_parse() {
    let buf = make_wav_buf(1000, 16000);
    let header = parse_wav_header(&buf).unwrap();
    assert_eq!(header.channels, 1);
    assert_eq!(header.sample_rate, 16000);
    assert_eq!(header.bits_per_sample, 16);
    assert_eq!(header.num_samples, 1000);
}

// Test 2: WAV parse rejects truncated data.
#[test]
fn test_wav_parse_rejects_truncated() {
    let buf = vec![0x52, 0x49, 0x46, 0x46, 0x00]; // RIFF but truncated
    let result = parse_wav_header(&buf);
    assert!(matches!(result, Err(VoiceError::BufferTooShort { .. })));
}

// Test 3: WAV parse rejects invalid RIFF signature.
#[test]
fn test_wav_parse_rejects_invalid_riff() {
    let mut buf = make_wav_buf(100, 16000);
    buf[0] = 0xFF;
    let result = parse_wav_header(&buf);
    assert!(matches!(result, Err(VoiceError::InvalidRiff)));
}

// Test 4: serialize/deserialize embedding round-trip.
#[test]
fn test_embedding_round_trip() {
    let original = vec![0.1f32, -0.5, 0.9, 1.0, -1.0, 0.0];
    let bytes = serialize_embedding(&original);
    let recovered = deserialize_embedding(&bytes).unwrap();
    assert_eq!(original.len(), recovered.len());
    for (a, b) in original.iter().zip(recovered.iter()) {
        assert!((a - b).abs() < 1e-6, "mismatch: {a} vs {b}");
    }
}

// Test 5: MFCC extraction returns correct frame count.
#[test]
fn test_mfcc_frame_count() {
    let sample_rate = 16000u32;
    // 1 second of silence -> expect ~99 frames (10ms step, 25ms frame)
    let pcm: Vec<i16> = vec![0i16; sample_rate as usize];
    let frames = extract_mfcc(&pcm, sample_rate, 13);
    // (16000 - 400) / 160 + 1 = 98 + 1 = 99
    assert!(
        frames.len() >= 90 && frames.len() <= 110,
        "unexpected frame count: {}",
        frames.len()
    );
    for frame in &frames {
        assert_eq!(frame.len(), 13);
    }
}

// Test 6: FFT of a pure sine wave has energy at correct bin.
#[test]
fn test_fft_sine_energy_at_correct_bin() {
    let n = 64usize;
    let freq_bin = 4usize; // expect energy at bin 4
    let mut data: Vec<(f64, f64)> = (0..n)
        .map(|k| {
            let angle = 2.0 * std::f64::consts::PI * freq_bin as f64 * k as f64 / n as f64;
            (angle.cos(), 0.0)
        })
        .collect();
    fft_inplace(&mut data);
    // bin 4 should have highest magnitude
    let magnitudes: Vec<f64> = data
        .iter()
        .map(|(re, im)| (re * re + im * im).sqrt())
        .collect();
    let max_bin = magnitudes
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    assert!(
        max_bin == freq_bin || max_bin == n - freq_bin,
        "expected peak at bin {freq_bin} or {}, got {max_bin}",
        n - freq_bin
    );
}

// Test 7: cosine_similarity of identical vectors = 1.0.
#[test]
fn test_cosine_identical_is_one() {
    let v = vec![0.3f32, 0.5, -0.2, 0.8, 0.1];
    let sim = cosine_similarity(&v, &v);
    assert!((sim - 1.0).abs() < 1e-5, "expected 1.0, got {sim}");
}

// Test 8: cosine_similarity of orthogonal vectors = 0.0.
#[test]
fn test_cosine_orthogonal_is_zero() {
    let a = vec![1.0f32, 0.0];
    let b = vec![0.0f32, 1.0];
    let sim = cosine_similarity(&a, &b);
    assert!(sim.abs() < 1e-5, "expected 0.0, got {sim}");
}

// Test 9: dtw_distance of identical sequences = 0.0.
#[test]
fn test_dtw_identical_is_zero() {
    let frames: Vec<Vec<f32>> = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
    let dist = dtw_distance(&frames, &frames);
    assert!(dist < 1e-5, "expected 0.0, got {dist}");
}

// Test 10: dtw_distance smaller for similar sequences than dissimilar.
#[test]
fn test_dtw_similar_smaller_than_dissimilar() {
    let probe = vec![vec![1.0f32, 1.0], vec![2.0, 2.0]];
    let similar = vec![vec![1.1f32, 1.1], vec![2.1, 2.1]];
    let dissimilar = vec![vec![10.0f32, 10.0], vec![20.0, 20.0]];
    let d_sim = dtw_distance(&probe, &similar);
    let d_dis = dtw_distance(&probe, &dissimilar);
    assert!(
        d_sim < d_dis,
        "similar distance {d_sim} should be less than dissimilar {d_dis}"
    );
}

// Test 11: match_voice_cosine genuine pair > threshold.
#[test]
fn test_match_cosine_genuine_above_threshold() {
    let emb = vec![0.6f32, 0.8, 0.0];
    let probe = make_template_with_embedding(emb.clone(), vec![]);
    let gallery = make_template_with_embedding(emb, vec![]);
    let score = match_voice_cosine(&probe, &gallery);
    assert!(
        score.raw > DEFAULT_THRESHOLD_COSINE,
        "genuine cosine score should exceed threshold: {}",
        score.raw
    );
}

// Test 12: match_voice_cosine imposter pair < threshold.
#[test]
fn test_match_cosine_imposter_below_threshold() {
    let probe_emb = vec![1.0f32, 0.0, 0.0];
    let gallery_emb = vec![0.0f32, 0.0, -1.0];
    let probe = make_template_with_embedding(probe_emb, vec![]);
    let gallery = make_template_with_embedding(gallery_emb, vec![]);
    let score = match_voice_cosine(&probe, &gallery);
    assert!(
        score.raw < DEFAULT_THRESHOLD_COSINE,
        "imposter cosine score should be below threshold: {}",
        score.raw
    );
}

// Test 13: quality SNR=30dB maps to high score.
#[test]
fn test_quality_high_snr_high_score() {
    let t = VoiceTemplate {
        template_id: "q-test".into(),
        utterance_type: UtteranceType::Passphrase,
        sample_rate_hz: 16000,
        duration_ms: 4000,
        language: "en".into(),
        embedding_dim: 0,
        embedding: vec![],
        mfcc_features: vec![],
        snr_db: Some(30.0),
        captured_at: Utc::now(),
    };
    let report = score_quality(&t);
    assert!(
        report.snr_score > 50.0,
        "SNR 30dB should give score > 50, got {}",
        report.snr_score
    );
}

// Test 14: quality zero duration returns 0.0.
#[test]
fn test_quality_zero_duration_returns_zero() {
    let t = VoiceTemplate {
        template_id: "q-zero".into(),
        utterance_type: UtteranceType::Passphrase,
        sample_rate_hz: 16000,
        duration_ms: 0,
        language: "en".into(),
        embedding_dim: 0,
        embedding: vec![],
        mfcc_features: vec![],
        snr_db: None,
        captured_at: Utc::now(),
    };
    let report = score_quality(&t);
    assert_eq!(report.overall, 0.0);
    assert_eq!(report.duration_score, 0.0);
}

// Test 15: quality short duration maps to low score.
#[test]
fn test_quality_short_duration_low_score() {
    let short = VoiceTemplate {
        template_id: "q-short".into(),
        utterance_type: UtteranceType::FreeSpeech,
        sample_rate_hz: 16000,
        duration_ms: 300, // under 500ms threshold
        language: "en".into(),
        embedding_dim: 0,
        embedding: vec![],
        mfcc_features: vec![],
        snr_db: Some(20.0),
        captured_at: Utc::now(),
    };
    let report = score_quality(&short);
    assert!(
        report.duration_score < 50.0,
        "short duration should yield low duration score, got {}",
        report.duration_score
    );
}
