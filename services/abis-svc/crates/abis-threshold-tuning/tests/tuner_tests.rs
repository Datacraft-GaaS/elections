//! Tests for the ROC computation and threshold selection logic.

use eel_abis_threshold_tuning::api::{simulate, SimulateRequest};
use eel_abis_threshold_tuning::domain::LabelledPair;
use eel_abis_threshold_tuning::tuner::{compute_roc, select_threshold, ThresholdTuner};
use eel_abis_threshold_tuning::ThresholdError;

fn synthetic_pairs(n: usize, sep: f64) -> Vec<LabelledPair> {
    let mut pairs = Vec::with_capacity(n * 2);
    let mut lcg: u64 = 42;
    let lcg_next = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*s as f64) / (u64::MAX as f64)
    };
    for _ in 0..n {
        let g = (lcg_next(&mut lcg) * 0.1 + 0.8 + sep).min(1.0);
        pairs.push(LabelledPair {
            score: g,
            genuine: true,
        });
        let i = (lcg_next(&mut lcg) * 0.1 + 0.2 - sep).max(0.0);
        pairs.push(LabelledPair {
            score: i,
            genuine: false,
        });
    }
    pairs
}

#[test]
fn roc_non_empty_for_valid_pairs() {
    let pairs = synthetic_pairs(50, 0.0);
    let roc = compute_roc(&pairs);
    assert!(!roc.is_empty());
}

#[test]
fn roc_fmr_decreases_with_threshold() {
    let pairs = synthetic_pairs(100, 0.1);
    let roc = compute_roc(&pairs);
    for w in roc.windows(2) {
        assert!(w[0].threshold <= w[1].threshold);
        assert!(w[0].fmr >= w[1].fmr - 1e-6);
    }
}

#[test]
fn select_threshold_respects_target_fmr() {
    let pairs = synthetic_pairs(200, 0.15);
    let roc = compute_roc(&pairs);
    let target = 0.01;
    let pt = select_threshold(&roc, target).unwrap();
    assert!(
        pt.fmr <= target + 1e-6,
        "FMR {:.4} exceeds target {}",
        pt.fmr,
        target
    );
}

#[test]
fn select_threshold_rejects_invalid_target() {
    let pairs = synthetic_pairs(50, 0.0);
    let roc = compute_roc(&pairs);
    assert!(matches!(
        select_threshold(&roc, -0.1),
        Err(ThresholdError::InvalidTargetFmr(_))
    ));
    assert!(matches!(
        select_threshold(&roc, 1.5),
        Err(ThresholdError::InvalidTargetFmr(_))
    ));
}

#[test]
fn tuner_insufficient_data_error() {
    let pairs = synthetic_pairs(5, 0.0);
    let tuner = ThresholdTuner::default();
    assert!(matches!(
        tuner.tune(&pairs, 0.001),
        Err(ThresholdError::InsufficientData { .. })
    ));
}

#[test]
fn tuner_produces_ci_monotonicity() {
    let pairs = synthetic_pairs(200, 0.2);
    let tuner = ThresholdTuner::new(200);
    let result = tuner.tune(&pairs, 0.01).unwrap();
    assert!(result.ci.lower <= result.threshold + 1e-6);
    assert!(result.ci.upper >= result.threshold - 1e-6);
}

#[test]
fn tuner_ci_lower_lte_upper() {
    let pairs = synthetic_pairs(100, 0.1);
    let tuner = ThresholdTuner::new(100);
    let result = tuner.tune(&pairs, 0.05).unwrap();
    assert!(result.ci.lower <= result.ci.upper);
}

#[test]
fn tuner_fmr_at_selected_threshold() {
    let pairs = synthetic_pairs(300, 0.2);
    let tuner = ThresholdTuner::new(50);
    let result = tuner.tune(&pairs, 0.01).unwrap();
    assert!(result.achieved_fmr <= 0.01 + 1e-6);
}

#[test]
fn tuner_fnmr_between_0_and_1() {
    let pairs = synthetic_pairs(200, 0.1);
    let tuner = ThresholdTuner::new(50);
    let result = tuner.tune(&pairs, 0.05).unwrap();
    assert!((0.0..=1.0).contains(&result.achieved_fnmr));
}

#[test]
fn tuner_n_pairs_reported_correctly() {
    let pairs = synthetic_pairs(100, 0.1);
    let tuner = ThresholdTuner::new(50);
    let result = tuner.tune(&pairs, 0.05).unwrap();
    assert_eq!(result.n_pairs, 200);
}

#[test]
fn roc_empty_for_empty_input() {
    let roc = compute_roc(&[]);
    assert!(roc.is_empty());
}

#[test]
fn roc_handles_all_genuine() {
    let pairs: Vec<LabelledPair> = (0..10)
        .map(|i| LabelledPair {
            score: i as f64 / 10.0,
            genuine: true,
        })
        .collect();
    let roc = compute_roc(&pairs);
    assert!(roc.is_empty());
}

#[test]
fn tuner_default_n_bootstrap_is_1000() {
    let t = ThresholdTuner::default();
    assert_eq!(t.n_bootstrap, 1000);
}

#[test]
fn simulate_api_returns_result() {
    let pairs = synthetic_pairs(100, 0.15);
    let req = SimulateRequest {
        pairs,
        target_fmr: 0.01,
        n_bootstrap: Some(50),
    };
    let result = simulate(req).unwrap();
    assert!(result.threshold > 0.0);
}

#[test]
fn bootstrap_reproducible_same_seed() {
    let pairs = synthetic_pairs(200, 0.15);
    let tuner = ThresholdTuner::new(100);
    let r1 = tuner.tune(&pairs, 0.01).unwrap();
    let r2 = tuner.tune(&pairs, 0.01).unwrap();
    assert!((r1.threshold - r2.threshold).abs() < 1e-10);
    assert!((r1.ci.lower - r2.ci.lower).abs() < 1e-10);
}
