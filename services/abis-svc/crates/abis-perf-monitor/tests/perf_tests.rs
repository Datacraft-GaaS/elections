//! Tests for performance monitoring.

use chrono::Utc;
use eel_abis_perf_monitor::domain::{HoldoutPair, PerformanceTarget, RollupPeriod};
use eel_abis_perf_monitor::error::PerfMonitorError;
use eel_abis_perf_monitor::pmr::{compute_fmr, compute_fnmr, compute_rank_n_accuracy, run_pmr};
use eel_abis_perf_monitor::report::build_report;
use eel_abis_perf_monitor::target::{all_targets_met, compare_to_target};
use uuid::Uuid;

fn pair(score: f64, genuine: bool, rank: Option<usize>) -> HoldoutPair {
    HoldoutPair {
        probe_id: Uuid::now_v7(),
        gallery_id: Uuid::now_v7(),
        score,
        genuine,
        rank,
    }
}

fn synthetic_holdout(n_each: usize) -> Vec<HoldoutPair> {
    let mut pairs = Vec::with_capacity(n_each * 2);
    for i in 0..n_each {
        pairs.push(pair(0.85 + (i % 10) as f64 * 0.01, true, Some((i % 5) + 1)));
        pairs.push(pair(0.20 + (i % 10) as f64 * 0.01, false, None));
    }
    pairs
}

#[test]
fn fmr_zero_when_impostors_below_threshold() {
    let fmr = compute_fmr(&synthetic_holdout(50), 0.80).unwrap();
    assert_eq!(fmr, 0.0);
}

#[test]
fn fnmr_zero_when_genuines_above_threshold() {
    let fnmr = compute_fnmr(&synthetic_holdout(50), 0.50).unwrap();
    assert_eq!(fnmr, 0.0);
}

#[test]
fn rank1_accuracy_correct() {
    let pairs = vec![
        pair(0.9, true, Some(1)),
        pair(0.8, true, Some(2)),
        pair(0.7, true, Some(1)),
    ];
    let acc = compute_rank_n_accuracy(&pairs, 1);
    assert!((acc - 2.0 / 3.0).abs() < 1e-6);
}

#[test]
fn rank5_accuracy_counts_rank_lte_5() {
    let pairs: Vec<HoldoutPair> = (1..=10).map(|r| pair(0.9, true, Some(r))).collect();
    let acc = compute_rank_n_accuracy(&pairs, 5);
    assert!((acc - 0.5).abs() < 1e-6);
}

#[test]
fn pmr_insufficient_data_errors() {
    assert!(matches!(
        run_pmr(&synthetic_holdout(10), 0.5),
        Err(PerfMonitorError::InsufficientData { .. })
    ));
}

#[test]
fn pmr_runs_successfully() {
    let result = run_pmr(&synthetic_holdout(50), 0.75).unwrap();
    assert!((0.0..=1.0).contains(&result.fmr));
    assert!((0.0..=1.0).contains(&result.fnmr));
}

#[test]
fn pmr_invalid_threshold_negative() {
    assert!(matches!(
        run_pmr(&synthetic_holdout(50), -0.1),
        Err(PerfMonitorError::InvalidThreshold(_))
    ));
}

#[test]
fn pmr_invalid_threshold_over_one() {
    assert!(matches!(
        run_pmr(&synthetic_holdout(50), 1.1),
        Err(PerfMonitorError::InvalidThreshold(_))
    ));
}

#[test]
fn target_comparison_all_met() {
    let result = run_pmr(&synthetic_holdout(50), 0.75).unwrap();
    let target = PerformanceTarget {
        fmr_target: 1.0,
        fnmr_target: 1.0,
        rank1_target: 0.0,
    };
    assert!(all_targets_met(&compare_to_target(&result, &target)));
}

#[test]
fn target_comparison_fmr_breach() {
    // threshold 0.15 is below all impostor scores (0.20-0.29), so FMR = 1.0 > target 0.0.
    let result = run_pmr(&synthetic_holdout(50), 0.15).unwrap();
    let target = PerformanceTarget {
        fmr_target: 0.0,
        fnmr_target: 1.0,
        rank1_target: 0.0,
    };
    assert!(!compare_to_target(&result, &target).fmr_ok);
}

#[test]
fn report_built_correctly() {
    let result = run_pmr(&synthetic_holdout(50), 0.75).unwrap();
    let model_id = Uuid::now_v7();
    let now = Utc::now();
    let report = build_report(model_id, RollupPeriod::Daily, now, now, &result, 0.75);
    assert_eq!(report.model_id, model_id);
    assert_eq!(report.period, RollupPeriod::Daily);
    assert!((0.0..=1.0).contains(&report.fmr));
}

#[test]
fn fmr_invalid_threshold_errors() {
    assert!(compute_fmr(&synthetic_holdout(10), 1.5).is_err());
}

#[test]
fn rank_n_empty_returns_zero() {
    assert_eq!(compute_rank_n_accuracy(&[], 1), 0.0);
}
