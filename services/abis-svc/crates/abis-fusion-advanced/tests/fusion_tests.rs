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

use abis_fusion_advanced::cohort::CohortNormalizer;
use abis_fusion_advanced::context::{Location, MatchCandidate, SpatiotemporalFilter};
use abis_fusion_advanced::decision_level::*;
use abis_fusion_advanced::score_level::*;
use abis_fusion_advanced::FusionError;

#[test]
fn min_max_normalize_basic() {
    let result = min_max_normalize(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
    let expected = [0.0, 0.25, 0.5, 0.75, 1.0];
    for (a, b) in result.iter().zip(expected.iter()) {
        assert!((a - b).abs() < 1e-10, "got {a}, expected {b}");
    }
}

#[test]
fn min_max_normalize_uniform() {
    let result = min_max_normalize(&[3.0, 3.0, 3.0]).unwrap();
    assert!(result.iter().all(|&v| (v - 0.5).abs() < 1e-10));
}

#[test]
fn min_max_normalize_empty_returns_error() {
    assert!(matches!(
        min_max_normalize(&[]),
        Err(FusionError::EmptyScores)
    ));
}

#[test]
fn z_score_normalize_values() {
    let scores = [0.0, 1.0, 2.0, 3.0, 4.0];
    let result = z_score_normalize(&scores).unwrap();
    let std = 2.0_f64.sqrt();
    let expected = [-2.0 / std, -1.0 / std, 0.0, 1.0 / std, 2.0 / std];
    for (a, b) in result.iter().zip(expected.iter()) {
        assert!((a - b).abs() < 1e-9, "got {a}, expected {b}");
    }
}

#[test]
fn tanh_normalize_monotonic() {
    let scores = [1.0, 2.0, 3.0, 4.0, 5.0];
    let result = tanh_normalize(&scores).unwrap();
    for i in 0..result.len() - 1 {
        assert!(result[i] <= result[i + 1], "not monotonic at {i}");
    }
}

#[test]
fn double_sigmoid_normalize_in_range() {
    let scores = [-100.0, -1.0, 0.0, 1.0, 100.0];
    let result = double_sigmoid_normalize(&scores).unwrap();
    for &v in &result {
        assert!(v >= 0.0 && v <= 1.0, "value {v} out of [0,1]");
    }
}

#[test]
fn t_norm_value() {
    let v = t_norm(0.8, 0.9);
    assert!((v - 0.72).abs() < 1e-10);
}

#[test]
fn s_norm_value() {
    let v = s_norm(0.8, 0.9);
    assert!((v - 0.98).abs() < 1e-10);
}

#[test]
fn weighted_sum_basic() {
    let result = weighted_sum(&[0.5, 0.7], &[1.0, 2.0]).unwrap();
    let expected = (0.5 + 1.4) / 3.0;
    assert!((result - expected).abs() < 1e-10);
}

#[test]
fn weighted_sum_mismatched_lengths_error() {
    assert!(matches!(
        weighted_sum(&[0.5, 0.7, 0.9], &[1.0, 2.0]),
        Err(FusionError::DimensionMismatch { .. })
    ));
}

#[test]
fn product_fusion_basic() {
    let result = product_fusion(&[0.5, 0.4, 0.8]).unwrap();
    assert!((result - 0.16).abs() < 1e-10);
}

#[test]
fn likelihood_ratio_fusion_single() {
    let result = likelihood_ratio_fusion(&[0.9], &[0.5]).unwrap();
    assert!(result > 0.5, "expected > 0.5, got {result}");
}

#[test]
fn majority_vote_true() {
    assert_eq!(majority_vote(&[true, true, false]).unwrap(), true);
}

#[test]
fn majority_vote_false() {
    assert_eq!(majority_vote(&[false, false, true]).unwrap(), false);
}

#[test]
fn majority_vote_empty_error() {
    assert!(matches!(majority_vote(&[]), Err(FusionError::EmptyScores)));
}

#[test]
fn weighted_vote_true() {
    assert_eq!(weighted_vote(&[true, false], &[0.9, 0.1]).unwrap(), true);
}

#[test]
fn borda_count_two_voters() {
    let result = borda_count(&[vec![0, 1, 2], vec![1, 0, 2]]).unwrap();
    let pos_of_2 = result.iter().position(|&x| x == 2).unwrap();
    let pos_0 = result.iter().position(|&x| x == 0).unwrap();
    let pos_1 = result.iter().position(|&x| x == 1).unwrap();
    assert!(pos_0 < pos_of_2);
    assert!(pos_1 < pos_of_2);
}

#[test]
fn bayesian_belief_strong_evidence() {
    let result = bayesian_belief(&[0.9, 0.8], 0.5).unwrap();
    assert!(result > 0.9, "expected > 0.9, got {result}");
}

#[test]
fn cohort_normalizer_basic() {
    let mut normalizer = CohortNormalizer::new();
    normalizer.add_cohort("adult-male".into(), 0.6, 0.1, 1000);
    let result = normalizer.normalize("adult-male", 0.7).unwrap();
    assert!((result - 1.0).abs() < 1e-10);
}

#[test]
fn cohort_normalizer_unknown_cohort_error() {
    let normalizer = CohortNormalizer::new();
    assert!(matches!(
        normalizer.normalize("unknown", 0.5),
        Err(FusionError::InsufficientCohortData(_))
    ));
}

#[test]
fn spatiotemporal_filter_passes_nearby() {
    let filter = SpatiotemporalFilter::new(100.0, 3600);
    let probe = Location { lat: 0.0, lon: 0.0 };
    let candidates = vec![MatchCandidate {
        id: "a".into(),
        station: Location { lat: 0.0, lon: 0.0 },
        timestamp_utc: 0,
        score: 0.9,
    }];
    let result = filter.filter(&probe, 0, &candidates);
    assert_eq!(result.len(), 1);
}

#[test]
fn spatiotemporal_filter_excludes_far() {
    let filter = SpatiotemporalFilter::new(10.0, 3600);
    let probe = Location { lat: 0.0, lon: 0.0 };
    let candidates = vec![MatchCandidate {
        id: "b".into(),
        station: Location {
            lat: 10.0,
            lon: 10.0,
        },
        timestamp_utc: 0,
        score: 0.9,
    }];
    let result = filter.filter(&probe, 0, &candidates);
    assert_eq!(result.len(), 0);
}
