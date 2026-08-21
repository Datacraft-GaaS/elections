use eel_abis_face_embed::{
    cosine_distance, cosine_similarity, is_normalised, l2_normalise, l2_normalise_slice,
    EmbedError, EmbeddingModel, FaceEmbedder, HashEmbedder, LossFamily, EMBED_DIM,
};

fn dummy_image(seed: u8) -> Vec<u8> {
    vec![seed; 112 * 112 * 3]
}

// --- L2 normalisation tests ---

#[test]
fn l2_normalise_produces_unit_norm() {
    let mut v = [0f32; EMBED_DIM];
    v[0] = 3.0;
    v[1] = 4.0;
    l2_normalise(&mut v);
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-5, "norm={}", norm);
}

#[test]
fn l2_normalise_zero_vector_unchanged() {
    let mut v = [0f32; EMBED_DIM];
    l2_normalise(&mut v);
    assert_eq!(v[0], 0.0);
}

#[test]
fn l2_normalise_slice_unit_norm() {
    let v: Vec<f32> = (0..16).map(|i| i as f32).collect();
    let normed = l2_normalise_slice(&v);
    let norm: f32 = normed.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-5);
}

#[test]
fn is_normalised_true_for_unit_vector() {
    let mut v = [0f32; EMBED_DIM];
    v[0] = 1.0;
    assert!(is_normalised(&v));
}

#[test]
fn is_normalised_false_for_unnormalised() {
    let v = vec![2.0f32; EMBED_DIM];
    assert!(!is_normalised(&v));
}

// --- Cosine similarity tests ---

#[test]
fn cosine_similarity_identical_unit_vectors() {
    let mut a = [0f32; EMBED_DIM];
    a[0] = 1.0;
    let b = a;
    assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-5);
}

#[test]
fn cosine_distance_identical_is_zero() {
    let mut a = [0f32; EMBED_DIM];
    a[0] = 1.0;
    assert!(cosine_distance(&a[..], &a[..]) < 1e-5);
}

#[test]
fn cosine_similarity_orthogonal_vectors() {
    let mut a = [0f32; EMBED_DIM];
    let mut b = [0f32; EMBED_DIM];
    a[0] = 1.0;
    b[1] = 1.0;
    let sim = cosine_similarity(&a, &b);
    assert!(sim.abs() < 1e-5);
}

// --- HashEmbedder tests ---

#[test]
fn hash_embedder_empty_input_errors() {
    let emb = HashEmbedder::new();
    assert!(matches!(emb.embed(&[]), Err(EmbedError::InvalidInput)));
}

#[test]
fn hash_embedder_identical_inputs_identical_output() {
    let emb = HashEmbedder::new();
    let img = dummy_image(42);
    let a = emb.embed(&img).unwrap();
    let b = emb.embed(&img).unwrap();
    assert_eq!(a, b);
}

#[test]
fn hash_embedder_different_inputs_different_output() {
    let emb = HashEmbedder::new();
    let a = emb.embed(&dummy_image(10)).unwrap();
    let b = emb.embed(&dummy_image(20)).unwrap();
    assert_ne!(a, b);
}

#[test]
fn hash_embedder_output_l2_normalised() {
    let emb = HashEmbedder::new();
    let v = emb.embed(&dummy_image(1)).unwrap();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "norm={}", norm);
}

#[test]
fn hash_embedder_different_inputs_low_cosine_similarity() {
    // Distinct seeds should produce embeddings with cosine < 0.4.
    let emb = HashEmbedder::new();
    let mut failures = 0;
    for seed_a in 0u8..8 {
        for seed_b in (seed_a + 1)..9 {
            let a = emb.embed(&dummy_image(seed_a)).unwrap();
            let b = emb.embed(&dummy_image(seed_b)).unwrap();
            if cosine_similarity(&a, &b) >= 0.4 {
                failures += 1;
            }
        }
    }
    assert_eq!(failures, 0, "hash collisions detected");
}

#[test]
fn hash_embedder_output_length() {
    let emb = HashEmbedder::new();
    let v = emb.embed(&dummy_image(5)).unwrap();
    assert_eq!(v.len(), EMBED_DIM);
}

#[test]
fn embed_vec_same_as_embed() {
    let emb = HashEmbedder::new();
    let img = dummy_image(7);
    let arr = emb.embed(&img).unwrap();
    let vec = emb.embed_vec(&img).unwrap();
    assert_eq!(arr.to_vec(), vec);
}

// --- EmbeddingModel / LossFamily tests ---

#[test]
fn embedding_model_arcface_family() {
    let m = EmbeddingModel::arcface("/models/arcface.onnx");
    assert_eq!(m.family, LossFamily::ArcFace);
    assert!(m.onnx_path.is_some());
}

#[test]
fn embedding_model_adaface_family() {
    let m = EmbeddingModel::adaface("/models/adaface.onnx");
    assert_eq!(m.family, LossFamily::AdaFace);
}

#[test]
fn embedding_model_cosface_family() {
    let m = EmbeddingModel::cosface("/models/cosface.onnx");
    assert_eq!(m.family, LossFamily::CosFace);
}

#[test]
fn embedding_model_baseline_no_path() {
    let m = EmbeddingModel::baseline(LossFamily::ArcFace);
    assert!(m.onnx_path.is_none());
}

#[test]
fn loss_family_all_variants_distinct() {
    let families = [
        LossFamily::ArcFace,
        LossFamily::AdaFace,
        LossFamily::CosFace,
    ];
    for i in 0..families.len() {
        for j in 0..families.len() {
            if i != j {
                assert_ne!(families[i], families[j]);
            }
        }
    }
}
