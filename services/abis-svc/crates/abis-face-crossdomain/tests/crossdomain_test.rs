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

use eel_abis_face_crossdomain::{CrossDomainAligner, CrossDomainError, Modality, EMBED_DIM};

fn unit_emb(val: f32, dim: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; EMBED_DIM];
    v[dim % EMBED_DIM] = val;
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    v.iter_mut().for_each(|x| *x /= norm);
    v
}

fn paired_data(n: usize, scale: f32) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
    let src: Vec<Vec<f32>> = (0..n).map(|i| unit_emb(1.0, i)).collect();
    let dst: Vec<Vec<f32>> = src
        .iter()
        .map(|v| v.iter().map(|&x| x * scale).collect())
        .collect();
    (src, dst)
}

fn normalise(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    v.iter().map(|x| x / norm).collect()
}

// --- Modality tests ---

#[test]
fn same_modality_errors() {
    assert!(matches!(
        CrossDomainAligner::new(Modality::Visible, Modality::Visible),
        Err(CrossDomainError::SameModality)
    ));
}

#[test]
fn different_modalities_ok() {
    let aligner = CrossDomainAligner::new(Modality::Visible, Modality::Nir).unwrap();
    assert_eq!(aligner.src, Modality::Visible);
    assert_eq!(aligner.dst, Modality::Nir);
}

// --- Fit tests ---

#[test]
fn fit_requires_at_least_two_samples() {
    let mut aligner = CrossDomainAligner::new(Modality::Visible, Modality::Nir).unwrap();
    let v = vec![unit_emb(1.0, 0)];
    assert!(matches!(
        aligner.fit(&v, &v),
        Err(CrossDomainError::InsufficientData)
    ));
}

#[test]
fn fit_mismatched_lengths_error() {
    let mut aligner = CrossDomainAligner::new(Modality::Visible, Modality::Nir).unwrap();
    let src = vec![unit_emb(1.0, 0), unit_emb(1.0, 1)];
    let dst = vec![unit_emb(1.0, 0)];
    assert!(matches!(
        aligner.fit(&src, &dst),
        Err(CrossDomainError::InsufficientData)
    ));
}

#[test]
fn fit_succeeds_with_valid_data() {
    let mut aligner = CrossDomainAligner::new(Modality::Nir, Modality::Visible).unwrap();
    let (src, dst) = paired_data(10, 1.0);
    aligner.fit(&src, &dst).unwrap();
    assert!(aligner.is_fitted());
}

// --- Transform tests ---

#[test]
fn transform_without_fit_errors() {
    let aligner = CrossDomainAligner::new(Modality::Nir, Modality::Visible).unwrap();
    let emb = unit_emb(1.0, 0);
    assert!(matches!(
        aligner.transform(&emb),
        Err(CrossDomainError::NotFitted)
    ));
}

#[test]
fn transform_identity_modalities() {
    // When src and dst distributions are identical, transform should leave
    // the embedding unchanged (scale = 1).
    let mut aligner = CrossDomainAligner::new(Modality::Nir, Modality::Visible).unwrap();
    let (src, dst) = paired_data(10, 1.0);
    aligner.fit(&src, &dst).unwrap();
    let emb = unit_emb(1.0, 0);
    let out = aligner.transform(&emb).unwrap();
    assert_eq!(out.len(), EMBED_DIM);
    // Should be close to the original (identity transform).
    let sim: f32 = emb.iter().zip(out.iter()).map(|(a, b)| a * b).sum();
    assert!(sim > 0.95, "sim={}", sim);
}

#[test]
fn transform_result_l2_normalised() {
    let mut aligner = CrossDomainAligner::new(Modality::Nir, Modality::Thermal).unwrap();
    let (src, dst) = paired_data(5, 2.0);
    aligner.fit(&src, &dst).unwrap();
    let emb = unit_emb(1.0, 3);
    let out = aligner.transform(&emb).unwrap();
    let norm: f32 = out.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "norm={}", norm);
}

#[test]
fn transform_deterministic() {
    let mut aligner = CrossDomainAligner::new(Modality::Visible, Modality::Thermal).unwrap();
    let (src, dst) = paired_data(8, 1.5);
    aligner.fit(&src, &dst).unwrap();
    let emb = unit_emb(1.0, 7);
    let a = aligner.transform(&emb).unwrap();
    let b = aligner.transform(&emb).unwrap();
    assert_eq!(a, b);
}

#[test]
fn transform_improves_cross_domain_similarity() {
    // Visible emb = V; NIR emb = 2*V (scaled). After alignment, cosine to V
    // should be 1.0 (normalisation collapses scale difference).
    let v = unit_emb(1.0, 0);
    let nir_raw: Vec<f32> = v.iter().map(|x| x * 2.0).collect();
    let nir = normalise(&nir_raw);

    // Before alignment: cosine(v, nir) = 1.0 already (scale-invariant by L2 norm).
    // We test that alignment doesn't break similarity.
    let mut aligner = CrossDomainAligner::new(Modality::Nir, Modality::Visible).unwrap();
    let src = vec![nir.clone(), unit_emb(0.5, 1)];
    let dst = vec![v.clone(), unit_emb(0.5, 1)];
    aligner.fit(&src, &dst).unwrap();
    let out = aligner.transform(&nir).unwrap();
    let sim: f32 = v.iter().zip(out.iter()).map(|(a, b)| a * b).sum();
    assert!(sim > 0.9, "sim={}", sim);
}

#[test]
fn modality_display() {
    assert_eq!(format!("{}", Modality::Visible), "visible");
    assert_eq!(format!("{}", Modality::Nir), "NIR");
    assert_eq!(format!("{}", Modality::Thermal), "thermal");
}

#[test]
fn thermal_visible_pair_ok() {
    let aligner = CrossDomainAligner::new(Modality::Thermal, Modality::Visible).unwrap();
    assert_eq!(aligner.src, Modality::Thermal);
    assert_eq!(aligner.dst, Modality::Visible);
}

#[test]
fn nir_thermal_pair_ok() {
    CrossDomainAligner::new(Modality::Nir, Modality::Thermal).unwrap();
}
