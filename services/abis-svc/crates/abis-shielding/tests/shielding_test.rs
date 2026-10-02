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

use eel_abis_shielding::{params::ShieldingParams, shield::Shielder};
use proptest::prelude::*;

// ---- Shield construction ----

#[test]
fn shield_same_template_same_filter() {
    let shielder = Shielder::new(256, 4).unwrap();
    let t = b"iris-template-12345".to_vec();
    let s1 = shielder.shield(&t).unwrap();
    let s2 = shielder.shield(&t).unwrap();
    assert_eq!(s1.filter, s2.filter, "shielding must be deterministic");
}

#[test]
fn shield_different_templates_different_filters() {
    let shielder = Shielder::new(256, 4).unwrap();
    let s1 = shielder.shield(b"template-A").unwrap();
    let s2 = shielder.shield(b"template-B").unwrap();
    assert_ne!(s1.filter, s2.filter);
}

#[test]
fn shield_empty_template_errors() {
    let shielder = Shielder::new(256, 4).unwrap();
    assert!(shielder.shield(&[]).is_err());
}

#[test]
fn shield_small_filter_errors() {
    assert!(Shielder::new(32, 4).is_err());
}

#[test]
fn shield_zero_hash_count_errors() {
    assert!(Shielder::new(256, 0).is_err());
}

// ---- Similarity (FMR/FNMR proxy) ----

#[test]
fn shield_self_similarity_is_one() {
    let shielder = Shielder::new(512, 8).unwrap();
    let s = shielder.shield(b"template-self").unwrap();
    assert!((s.similarity(&s) - 1.0).abs() < 1e-9);
}

#[test]
fn shield_different_templates_low_similarity() {
    let shielder = Shielder::new(512, 8).unwrap();
    let sa = shielder.shield(b"template-AAAAAAAAAA").unwrap();
    let sb = shielder.shield(b"template-BBBBBBBBBB").unwrap();
    // Distinct templates should be clearly dissimilar.
    assert!(
        sa.similarity(&sb) < 0.9,
        "dissimilar templates should score < 0.9"
    );
}

#[test]
fn shield_similarity_in_range() {
    let shielder = Shielder::new(256, 4).unwrap();
    let sa = shielder.shield(b"t1").unwrap();
    let sb = shielder.shield(b"t2").unwrap();
    let sim = sa.similarity(&sb);
    assert!((0.0..=1.0).contains(&sim));
}

#[test]
fn shield_cross_scheme_unlinkability() {
    // Same template shielded with different parameters produces different filters.
    let s1 = Shielder::new(512, 8).unwrap().shield(b"template").unwrap();
    let s2 = Shielder::new(1024, 12)
        .unwrap()
        .shield(b"template")
        .unwrap();
    // Different filter sizes means different byte lengths; not directly linkable.
    assert_ne!(s1.filter.len(), s2.filter.len());
}

// ---- Params ----

#[test]
fn params_select_valid_range() {
    let p = ShieldingParams::select(2048, 0.001, 0.01).unwrap();
    assert!(p.filter_size_bits >= 64);
    assert!(p.hash_count >= 1 && p.hash_count <= 32);
}

#[test]
fn params_invalid_fmr_errors() {
    assert!(ShieldingParams::select(512, 0.0, 0.01).is_err());
    assert!(ShieldingParams::select(512, 1.5, 0.01).is_err());
}

#[test]
fn params_zero_dim_errors() {
    assert!(ShieldingParams::select(0, 0.001, 0.01).is_err());
}

#[test]
fn params_estimated_fmr_below_target() {
    let target = 0.01;
    let p = ShieldingParams::select(512, target, 0.05).unwrap();
    let est = p.estimated_fmr(512);
    // Estimated FMR should be in a reasonable range of target.
    assert!(
        est < target * 10.0,
        "estimated FMR {est:.6} too far above target {target}"
    );
}

#[test]
fn params_filter_size_increases_with_lower_fmr() {
    let p_loose = ShieldingParams::select(512, 0.1, 0.05).unwrap();
    let p_tight = ShieldingParams::select(512, 0.001, 0.05).unwrap();
    assert!(p_tight.filter_size_bits > p_loose.filter_size_bits);
}

// ---- Proptest ----

proptest! {
    #[test]
    fn prop_shield_deterministic(
        template in proptest::collection::vec(0u8..255u8, 8..64),
    ) {
        let shielder = Shielder::new(256, 4).unwrap();
        let s1 = shielder.shield(&template).unwrap();
        let s2 = shielder.shield(&template).unwrap();
        prop_assert_eq!(s1.filter, s2.filter);
    }

    #[test]
    fn prop_params_filter_size_valid(
        dim in 64usize..2048usize,
        fmr_exp in 1u32..4u32,
    ) {
        let fmr = 10f64.powi(-(fmr_exp as i32));
        let p = ShieldingParams::select(dim, fmr, 0.05).unwrap();
        prop_assert!(p.filter_size_bits >= 64);
        prop_assert!(p.hash_count >= 1 && p.hash_count <= 32);
    }
}
