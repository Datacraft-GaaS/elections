use eel_abis_face_deident::{
    blur_image, kanon::embedding_hash, pixelate_image, DpEmbeddingNoise, DpError, KAnonDeident,
    KAnonError,
};

const EMBED_DIM: usize = 512;

fn unit_emb(i: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; EMBED_DIM];
    v[i % EMBED_DIM] = 1.0;
    v
}

fn uniform_pixels(val: u8, n: usize) -> Vec<u8> {
    vec![val; n * 3]
}

fn build_anon_set(k: usize) -> Vec<(Vec<f32>, Vec<u8>)> {
    (1..=k)
        .map(|i| (unit_emb(i), uniform_pixels((i * 10) as u8, 112 * 112)))
        .collect()
}

// --- KAnonDeident tests ---

#[test]
fn kanon_k_less_than_2_errors() {
    assert!(matches!(KAnonDeident::new(1), Err(KAnonError::KTooSmall)));
}

#[test]
fn kanon_insufficient_anon_set_errors() {
    let d = KAnonDeident::new(5).unwrap();
    let target_emb = unit_emb(0);
    let target_px = uniform_pixels(200, 112 * 112);
    let anon = build_anon_set(3);
    assert!(matches!(
        d.deidentify(&target_emb, &target_px, &anon),
        Err(KAnonError::InsufficientAnonymitySet(_, _))
    ));
}

#[test]
fn kanon_output_not_original_identity() {
    let d = KAnonDeident::new(3).unwrap();
    let target_emb = unit_emb(0);
    let target_px = uniform_pixels(200, 112 * 112);
    let anon = build_anon_set(5);
    let out = d.deidentify(&target_emb, &target_px, &anon).unwrap();
    assert_ne!(
        embedding_hash(&out),
        embedding_hash(&target_px),
        "output should differ from original face"
    );
}

#[test]
fn kanon_output_same_length_as_input() {
    let d = KAnonDeident::new(3).unwrap();
    let target_emb = unit_emb(0);
    let target_px = uniform_pixels(180, 112 * 112);
    let anon = build_anon_set(5);
    let out = d.deidentify(&target_emb, &target_px, &anon).unwrap();
    assert_eq!(out.len(), target_px.len());
}

#[test]
fn kanon_buffer_mismatch_errors() {
    let d = KAnonDeident::new(2).unwrap();
    let target_emb = unit_emb(0);
    let target_px = vec![100u8; 10];
    // Anon set has k elements but wrong buffer length.
    let anon = vec![
        (unit_emb(1), vec![200u8; 20]),
        (unit_emb(2), vec![200u8; 20]),
    ];
    assert!(matches!(
        d.deidentify(&target_emb, &target_px, &anon),
        Err(KAnonError::BufferMismatch)
    ));
}

#[test]
fn kanon_composite_is_mean_of_k_neighbours() {
    // All anon pixels the same value => composite should match.
    let d = KAnonDeident::new(2).unwrap();
    let target_emb = unit_emb(0);
    let target_px = uniform_pixels(100, 4);
    let anon = vec![
        (unit_emb(1), uniform_pixels(50, 4)),
        (unit_emb(2), uniform_pixels(50, 4)),
        (unit_emb(3), uniform_pixels(50, 4)),
    ];
    let out = d.deidentify(&target_emb, &target_px, &anon).unwrap();
    // All output pixels should be 50 (mean of identical anon pixels).
    assert!(out.iter().all(|&v| v == 50), "out={:?}", &out[..6]);
}

// --- DpEmbeddingNoise tests ---

#[test]
fn dp_invalid_epsilon_errors() {
    assert!(matches!(
        DpEmbeddingNoise::new(0.0, 1e-5),
        Err(DpError::InvalidEpsilon)
    ));
    assert!(matches!(
        DpEmbeddingNoise::new(-1.0, 1e-5),
        Err(DpError::InvalidEpsilon)
    ));
}

#[test]
fn dp_invalid_delta_errors() {
    assert!(matches!(
        DpEmbeddingNoise::new(1.0, 0.0),
        Err(DpError::InvalidDelta)
    ));
    assert!(matches!(
        DpEmbeddingNoise::new(1.0, 1.0),
        Err(DpError::InvalidDelta)
    ));
}

#[test]
fn dp_noise_adds_perturbation() {
    let dp = DpEmbeddingNoise::new(1.0, 1e-5).unwrap();
    let emb = unit_emb(0);
    let noisy = dp.add_noise(&emb, 42).unwrap();
    // Should differ from original (noise added).
    assert_ne!(emb, noisy);
}

#[test]
fn dp_noise_output_l2_normalised() {
    let dp = DpEmbeddingNoise::default();
    let emb = unit_emb(5);
    let noisy = dp.add_noise(&emb, 7).unwrap();
    let norm: f32 = noisy.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "norm={}", norm);
}

#[test]
fn dp_sigma_positive() {
    let dp = DpEmbeddingNoise::default();
    assert!(dp.sigma() > 0.0);
}

// --- Blur tests ---

#[test]
fn blur_uniform_image_unchanged() {
    let px = uniform_pixels(128, 16);
    let out = blur_image(&px, 16, 16, 2);
    assert!(out.iter().all(|&v| (v as i32 - 128).abs() <= 1));
}

#[test]
fn blur_output_same_length() {
    let px = uniform_pixels(100, 32);
    let out = blur_image(&px, 32, 32, 3);
    assert_eq!(out.len(), px.len());
}

#[test]
fn blur_preserves_colour_distribution_roughly() {
    let px = uniform_pixels(200, 64);
    let out = blur_image(&px, 64, 64, 5);
    let mean: f32 = out.iter().map(|&v| v as f32).sum::<f32>() / out.len() as f32;
    assert!((mean - 200.0).abs() < 5.0, "mean={}", mean);
}

// --- Pixelation tests ---

#[test]
fn pixelate_uniform_image_unchanged() {
    let px = uniform_pixels(77, 16);
    let out = pixelate_image(&px, 16, 16, 4);
    assert!(out.iter().all(|&v| v == 77));
}

#[test]
fn pixelate_output_same_length() {
    let px = uniform_pixels(50, 32);
    let out = pixelate_image(&px, 32, 32, 8);
    assert_eq!(out.len(), px.len());
}
