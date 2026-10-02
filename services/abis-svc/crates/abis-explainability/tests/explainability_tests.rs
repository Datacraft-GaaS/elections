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

use abis_explainability::attention::{AttentionEmitter, UniformAttentionEmitter};
use abis_explainability::lime::{LimeExplainer, LocalLimeExplainer};
use abis_explainability::overlay::*;
use abis_explainability::reasoning::{generate_reasoning_text, ModalityResult};
use abis_explainability::saliency::*;
use abis_explainability::shap::{LocalShapExplainer, ShapExplainer};
use abis_explainability::ExplainError;

fn make_minutia(x: f32, y: f32, matched: bool) -> MinutiaPoint {
    MinutiaPoint {
        x,
        y,
        angle_deg: 45.0,
        quality: 0.9,
        is_matched: matched,
    }
}

fn make_config(w: u32, h: u32) -> OverlayConfig {
    OverlayConfig {
        width: w,
        height: h,
        scale: 1.0,
    }
}

#[test]
fn generate_svg_basic() {
    let probe = vec![
        make_minutia(10.0, 10.0, true),
        make_minutia(20.0, 20.0, false),
    ];
    let gallery = vec![
        make_minutia(15.0, 15.0, true),
        make_minutia(25.0, 25.0, false),
    ];
    let pairs = vec![MinutiaePair {
        probe_idx: 0,
        gallery_idx: 0,
        confidence: 0.9,
    }];
    let config = make_config(100, 100);
    let svg = generate_svg(&probe, &gallery, &pairs, &config).unwrap();
    assert!(svg.starts_with("<svg"));
}

#[test]
fn generate_svg_empty_probe_error() {
    let result = generate_svg(&[], &[], &[], &make_config(100, 100));
    assert!(matches!(result, Err(ExplainError::EmptyTemplate)));
}

#[test]
fn generate_svg_contains_confidence() {
    let probe = vec![make_minutia(10.0, 10.0, true)];
    let gallery = vec![make_minutia(10.0, 10.0, true)];
    let pairs = vec![MinutiaePair {
        probe_idx: 0,
        gallery_idx: 0,
        confidence: 0.85,
    }];
    let svg = generate_svg(&probe, &gallery, &pairs, &make_config(200, 200)).unwrap();
    assert!(
        svg.contains("confidence"),
        "SVG should contain confidence: {svg}"
    );
}

#[test]
fn compute_saliency_16dim_4x4() {
    let a: Vec<f32> = (0..16).map(|i| i as f32 * 0.1).collect();
    let b: Vec<f32> = (0..16).map(|i| i as f32 * 0.05).collect();
    let map = compute_saliency(&a, &b, 40, 40, 4).unwrap();
    assert_eq!(map.regions.len(), 16);
}

#[test]
fn compute_saliency_empty_error() {
    let result = compute_saliency(&[], &[1.0], 10, 10, 2);
    assert!(matches!(result, Err(ExplainError::InvalidInput(_))));
}

#[test]
fn compute_saliency_gradient_magnitude() {
    let a = vec![0.8f32, 0.2];
    let b = vec![0.3f32, 0.9];
    let map = compute_saliency(&a, &b, 20, 10, 2).unwrap();
    assert!((map.regions[0].gradient_magnitude - 0.5).abs() < 1e-5);
}

#[test]
fn shap_sums_to_one() {
    let explainer = LocalShapExplainer { n_samples: 100 };
    let fv = vec![0.5, 0.3, 0.8, 0.1];
    let bl = vec![0.2, 0.2, 0.2, 0.2];
    let shap_vals = explainer.explain(&fv, &bl).unwrap();
    let sum: f64 = shap_vals.iter().sum();
    assert!(
        (sum - 1.0).abs() < 1e-5 || (sum + 1.0).abs() < 1e-5,
        "sum was {sum}"
    );
}

#[test]
fn shap_empty_feature_error() {
    let explainer = LocalShapExplainer { n_samples: 10 };
    assert!(matches!(
        explainer.explain(&[], &[]),
        Err(ExplainError::InvalidInput(_))
    ));
}

#[test]
fn lime_output_len() {
    let explainer = LocalLimeExplainer { kernel_width: 0.25 };
    let fv = vec![0.1, 0.5, 0.9];
    let result = explainer.explain(&fv, 100).unwrap();
    assert_eq!(result.len(), fv.len());
}

#[test]
fn uniform_attention_sums_to_one() {
    let emitter = UniformAttentionEmitter;
    let map = emitter.emit_attention(10, 10).unwrap();
    let sum: f32 = map.values.iter().sum();
    assert!((sum - 1.0).abs() < 1e-5);
}

#[test]
fn uniform_attention_len() {
    let emitter = UniformAttentionEmitter;
    let map = emitter.emit_attention(8, 6).unwrap();
    assert_eq!(map.values.len(), 48);
}

#[test]
fn reasoning_match_output() {
    let results = vec![ModalityResult {
        modality: "Right Thumb".into(),
        matched_minutiae: 24,
        probe_total: 32,
        gallery_total: 30,
        fusion_score: 0.87,
        threshold: 0.72,
    }];
    let text = generate_reasoning_text(&results, true).unwrap();
    assert!(text.contains("MATCH"));
}

#[test]
fn reasoning_no_match_output() {
    let results = vec![ModalityResult {
        modality: "Left Index".into(),
        matched_minutiae: 5,
        probe_total: 30,
        gallery_total: 28,
        fusion_score: 0.4,
        threshold: 0.72,
    }];
    let text = generate_reasoning_text(&results, false).unwrap();
    assert!(text.contains("NO MATCH"));
}

#[test]
fn reasoning_empty_error() {
    assert!(matches!(
        generate_reasoning_text(&[], true),
        Err(ExplainError::EmptyTemplate)
    ));
}

#[test]
fn reasoning_contains_minutiae_counts() {
    let results = vec![ModalityResult {
        modality: "Right Index".into(),
        matched_minutiae: 15,
        probe_total: 25,
        gallery_total: 20,
        fusion_score: 0.8,
        threshold: 0.7,
    }];
    let text = generate_reasoning_text(&results, true).unwrap();
    assert!(text.contains("25"), "should contain probe total 25: {text}");
    assert!(
        text.contains("20"),
        "should contain gallery total 20: {text}"
    );
}
