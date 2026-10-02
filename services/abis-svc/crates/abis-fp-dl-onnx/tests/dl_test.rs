// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for the DL ONNX matcher crate.
//!
//! All tests use the deterministic hash-based pseudo-embedding path
//! (no ONNX model file required).

use eel_abis_fp_dl_onnx::{
    embed::DeepPrintEmbedder,
    fusion::{fit_weights, fuse, FusionWeights},
    matcher::{cosine_similarity, DlMatcher},
    pool::EmbedderPool,
    GrayImage, EMBED_DIM,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn solid(v: u8) -> GrayImage {
    GrayImage::new(vec![v; 64 * 64], 64, 64)
}

fn stripe() -> GrayImage {
    let mut pix = vec![200u8; 64 * 64];
    for y in 0..64u32 {
        if y % 10 < 5 {
            for x in 0..64u32 {
                pix[(y * 64 + x) as usize] = 30;
            }
        }
    }
    GrayImage::new(pix, 64, 64)
}

// ---------------------------------------------------------------------------
// Embedding
// ---------------------------------------------------------------------------

#[test]
fn pseudo_embedding_correct_dim() {
    let img = solid(128);
    let emb = DeepPrintEmbedder::from_env().embed(&img).unwrap();
    assert_eq!(emb.len(), EMBED_DIM);
}

#[test]
fn pseudo_embedding_is_unit_normalised() {
    let img = solid(128);
    let emb = DeepPrintEmbedder::from_env().embed(&img).unwrap();
    let norm: f32 = emb.iter().map(|v| v * v).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "embedding norm={norm} not unit");
}

#[test]
fn identical_images_have_identical_embeddings() {
    let img = stripe();
    let e1 = DeepPrintEmbedder::from_env().embed(&img).unwrap();
    let e2 = DeepPrintEmbedder::from_env().embed(&img).unwrap();
    assert_eq!(e1, e2, "identical images must produce identical embeddings");
}

#[test]
fn different_images_produce_different_embeddings() {
    let e1 = DeepPrintEmbedder::from_env().embed(&solid(50)).unwrap();
    let e2 = DeepPrintEmbedder::from_env().embed(&solid(200)).unwrap();
    assert_ne!(e1, e2);
}

#[test]
fn preprocess_output_size() {
    let img = solid(128);
    let out = DeepPrintEmbedder::preprocess(&img);
    assert_eq!(out.len(), 224 * 224, "preprocess output should be 224x224");
}

// ---------------------------------------------------------------------------
// Cosine similarity
// ---------------------------------------------------------------------------

#[test]
fn identical_embeddings_cosine_one() {
    let img = stripe();
    let emb = DeepPrintEmbedder::from_env().embed(&img).unwrap();
    let sim = cosine_similarity(&emb, &emb);
    assert!(
        (sim - 1.0).abs() < 1e-5,
        "identical embeddings cosine={sim}"
    );
}

#[test]
fn different_images_cosine_below_threshold() {
    let e1 = DeepPrintEmbedder::from_env().embed(&solid(30)).unwrap();
    let e2 = DeepPrintEmbedder::from_env().embed(&solid(230)).unwrap();
    let sim = cosine_similarity(&e1, &e2);
    assert!(
        sim < 0.5,
        "very different images cosine={sim} unexpectedly high"
    );
}

#[test]
fn cosine_bounded() {
    let e1 = DeepPrintEmbedder::from_env().embed(&solid(100)).unwrap();
    let e2 = DeepPrintEmbedder::from_env().embed(&stripe()).unwrap();
    let sim = cosine_similarity(&e1, &e2);
    assert!((0.0..=1.0).contains(&sim), "cosine={sim} out of [0,1]");
}

// ---------------------------------------------------------------------------
// DlMatcher
// ---------------------------------------------------------------------------

#[test]
fn matcher_compare_identical() {
    let img = stripe();
    let m = DlMatcher::default();
    let s = m.compare(&img, &img).unwrap();
    assert!((s - 1.0).abs() < 1e-5, "identical images matcher score={s}");
}

#[test]
fn matcher_compare_different_below_half() {
    let m = DlMatcher::default();
    let s = m.compare(&solid(20), &solid(235)).unwrap();
    assert!(s < 0.5, "very different images score={s}");
}

// ---------------------------------------------------------------------------
// Fusion
// ---------------------------------------------------------------------------

#[test]
fn fuse_equal_weights() {
    let s = fuse(0.8, 0.6, 0.4, &FusionWeights::default());
    let expected = (0.8 + 0.6 + 0.4) / 3.0;
    assert!((s - expected).abs() < 1e-5, "fused={s} expected={expected}");
}

#[test]
fn fuse_bounded() {
    let s = fuse(1.0, 1.0, 1.0, &FusionWeights::default());
    assert!((0.0..=1.0).contains(&s));
}

#[test]
fn fit_weights_returns_valid() {
    let samples = vec![
        (0.9f32, 0.8, 0.7, true),
        (0.2, 0.1, 0.15, false),
        (0.85, 0.75, 0.8, true),
        (0.1, 0.2, 0.05, false),
    ];
    let w = fit_weights(&samples);
    let total = w.dl + w.mcc + w.bozorth;
    assert!(
        (total - 1.0).abs() < 1e-4,
        "weights don't sum to 1: {total}"
    );
}

// ---------------------------------------------------------------------------
// Pool
// ---------------------------------------------------------------------------

#[test]
fn pool_serves_session() {
    let pool = EmbedderPool::new(4, None);
    let guard = pool.checkout().unwrap();
    let img = solid(128);
    let emb = guard.get().embed(&img).unwrap();
    assert_eq!(emb.len(), EMBED_DIM);
}

#[test]
fn pool_exhaustion_returns_error() {
    let pool = EmbedderPool::new(1, None);
    let _g1 = pool.checkout().unwrap();
    let result = pool.checkout();
    assert!(result.is_err(), "pool should return error when exhausted");
}

#[test]
fn pool_recycles_after_drop() {
    let pool = EmbedderPool::new(1, None);
    {
        let _g = pool.checkout().unwrap();
    }
    // After drop, pool should serve again.
    let g2 = pool.checkout();
    assert!(g2.is_ok(), "pool should serve after drop");
}
