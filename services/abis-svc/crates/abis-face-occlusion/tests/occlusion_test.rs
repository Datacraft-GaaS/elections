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

use eel_abis_face_occlusion::{
    OcclusionDetector, OcclusionRobustMatcher, OcclusionType, EMBED_DIM,
};

fn skin_rgb(w: u32, h: u32) -> Vec<u8> {
    let mut v = vec![0u8; (w * h * 3) as usize];
    for c in v.chunks_mut(3) {
        c[0] = 180;
        c[1] = 140;
        c[2] = 100;
    }
    v
}

fn masked_rgb(w: u32, h: u32) -> Vec<u8> {
    // Top 2/3 skin, bottom 1/3 dark (mask).
    let mut v = vec![0u8; (w * h * 3) as usize];
    let two_thirds = (h * 2 / 3) as usize * w as usize;
    for chunk in v[..two_thirds * 3].chunks_mut(3) {
        chunk[0] = 180;
        chunk[1] = 140;
        chunk[2] = 100;
    }
    v
}

fn unit_emb(i: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; EMBED_DIM];
    v[i % EMBED_DIM] = 1.0;
    v
}

// --- OcclusionDetector tests ---

#[test]
fn detect_empty_image_errors() {
    let det = OcclusionDetector::new();
    assert!(det.detect(&[], 112, 112).is_err());
}

#[test]
fn detect_all_skin_returns_none() {
    let det = OcclusionDetector::new();
    let rgb = skin_rgb(112, 112);
    let regions = det.detect(&rgb, 112, 112).unwrap();
    assert!(regions.iter().any(|r| r.region_type == OcclusionType::None));
}

#[test]
fn detect_masked_lower_face() {
    let det = OcclusionDetector::new();
    let rgb = masked_rgb(112, 112);
    let regions = det.detect(&rgb, 112, 112).unwrap();
    assert!(
        regions.iter().any(|r| r.region_type == OcclusionType::Mask),
        "regions: {:?}",
        regions.iter().map(|r| &r.region_type).collect::<Vec<_>>()
    );
}

#[test]
fn detect_dark_image_returns_multiple_regions() {
    let det = OcclusionDetector::new();
    // All-dark image: no skin anywhere => all three regions should flag.
    let rgb = vec![20u8; 112 * 112 * 3];
    let regions = det.detect(&rgb, 112, 112).unwrap();
    assert!(regions.len() > 1);
}

#[test]
fn occlusion_fraction_none_is_zero() {
    let det = OcclusionDetector::new();
    let rgb = skin_rgb(112, 112);
    let regions = det.detect(&rgb, 112, 112).unwrap();
    let frac = det.occlusion_fraction(&regions);
    assert!(frac < 0.1, "frac={}", frac);
}

#[test]
fn occlusion_fraction_bounded_0_1() {
    let det = OcclusionDetector::new();
    let rgb = vec![20u8; 112 * 112 * 3];
    let regions = det.detect(&rgb, 112, 112).unwrap();
    let frac = det.occlusion_fraction(&regions);
    assert!((0.0..=1.0).contains(&frac));
}

// --- OcclusionRobustMatcher tests ---

#[test]
fn gate_no_occlusion_all_ones() {
    let matcher = OcclusionRobustMatcher::default();
    let det = OcclusionDetector::new();
    let rgb = skin_rgb(112, 112);
    let regions = det.detect(&rgb, 112, 112).unwrap();
    let gate = matcher.build_gate(&regions);
    assert_eq!(gate.len(), EMBED_DIM);
    assert!(gate.iter().all(|&w| w == 1.0));
}

#[test]
fn gate_mask_reduces_lower_dims() {
    use eel_abis_face_occlusion::OcclusionRegion;
    let matcher = OcclusionRobustMatcher::default();
    let regions = vec![OcclusionRegion {
        region_type: OcclusionType::Mask,
        coverage: 0.8,
    }];
    let gate = matcher.build_gate(&regions);
    // Lower third of dims should be < 1.0.
    let seg = EMBED_DIM / 3;
    let upper_ok = gate[..seg].iter().all(|&w| (w - 1.0).abs() < 1e-5);
    let lower_reduced = gate[2 * seg..].iter().any(|&w| w < 1.0);
    assert!(upper_ok, "upper dims should be 1.0");
    assert!(lower_reduced, "lower dims should be reduced");
}

#[test]
fn masked_similarity_identical_no_occlusion() {
    let matcher = OcclusionRobustMatcher::default();
    let emb = unit_emb(0);
    let gate = vec![1.0f32; EMBED_DIM];
    let sim = matcher.masked_similarity(&emb, &emb, &gate).unwrap();
    assert!((sim - 1.0).abs() < 1e-4);
}

#[test]
fn masked_similarity_orthogonal_is_zero() {
    let matcher = OcclusionRobustMatcher::default();
    let a = unit_emb(0);
    let b = unit_emb(1);
    let gate = vec![1.0f32; EMBED_DIM];
    let sim = matcher.masked_similarity(&a, &b, &gate).unwrap();
    assert!(sim.abs() < 1e-4);
}

#[test]
fn masked_similarity_30pct_occlusion_identity_remains_high() {
    use eel_abis_face_occlusion::OcclusionRegion;
    let matcher = OcclusionRobustMatcher::default();
    // Same embedding for probe and gallery.
    let emb = {
        let mut v = vec![0.5f32; EMBED_DIM];
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter_mut().for_each(|x| *x /= norm);
        v
    };
    let regions = vec![OcclusionRegion {
        region_type: OcclusionType::Mask,
        coverage: 0.30,
    }];
    let gate = matcher.build_gate(&regions);
    let sim = matcher.masked_similarity(&emb, &emb, &gate).unwrap();
    assert!(sim > 0.9, "sim={}", sim);
}

#[test]
fn masked_similarity_dimension_mismatch_errors() {
    let matcher = OcclusionRobustMatcher::default();
    let emb = unit_emb(0);
    let short = vec![1.0f32; 10];
    let gate = vec![1.0f32; EMBED_DIM];
    assert!(matcher.masked_similarity(&short, &emb, &gate).is_err());
}

#[test]
fn hijab_reduces_false_rejects() {
    use eel_abis_face_occlusion::OcclusionRegion;
    let matcher = OcclusionRobustMatcher::default();
    // Two embeddings that are identical except upper-third noise due to hijab.
    let mut a = vec![0.0f32; EMBED_DIM];
    for (i, v) in a.iter_mut().enumerate() {
        *v = if i < EMBED_DIM / 3 { 0.01 } else { 1.0 };
    }
    let norm: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    a.iter_mut().for_each(|x| *x /= norm);
    let b = a.clone();
    let regions = vec![OcclusionRegion {
        region_type: OcclusionType::Hijab,
        coverage: 0.8,
    }];
    let gate = matcher.build_gate(&regions);
    let sim_gated = matcher.masked_similarity(&a, &b, &gate).unwrap();
    assert!(sim_gated > 0.9, "gated sim={}", sim_gated);
}
