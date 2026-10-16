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

//! Tests for drift detection statistics.

use chrono::Utc;
use eel_abis_drift_monitor::adaptive::suggest_adaptive_threshold;
use eel_abis_drift_monitor::alert::maybe_alert;
use eel_abis_drift_monitor::detector::{
    js_divergence, ks_statistic, psi, wasserstein_distance, DriftDetector,
};
use eel_abis_drift_monitor::domain::{
    DriftReport, DriftSeverity, DriftWindow, FeatureDistribution,
};
use uuid::Uuid;

fn uniform_dist(n: usize, total: u64) -> FeatureDistribution {
    FeatureDistribution {
        feature_name: "feat_a".to_string(),
        bin_counts: vec![total / n as u64; n],
        n_total: total,
        captured_at: Utc::now(),
    }
}

fn gaussian_approx(n: usize, total: u64) -> FeatureDistribution {
    let mut counts = Vec::with_capacity(n);
    for i in 0..n {
        let x = (i as f64 - n as f64 / 2.0) / (n as f64 / 4.0);
        let w = (-0.5 * x * x).exp();
        counts.push((w * total as f64 / n as f64) as u64 + 1);
    }
    FeatureDistribution {
        feature_name: "feat_b".to_string(),
        bin_counts: counts,
        n_total: total,
        captured_at: Utc::now(),
    }
}

#[test]
fn psi_zero_for_identical_distributions() {
    let d = uniform_dist(10, 1000);
    let p = d.to_probs();
    let val = psi(&p, &p).unwrap();
    assert!(
        val.abs() < 1e-6,
        "PSI should be ~0 for identical dists, got {val}"
    );
}

#[test]
fn psi_positive_for_different_distributions() {
    let base = uniform_dist(10, 1000);
    let curr = gaussian_approx(10, 1000);
    let p = base.to_probs();
    let q = curr.to_probs();
    let val = psi(&p, &q).unwrap();
    assert!(
        val > 0.0,
        "PSI should be positive for different distributions"
    );
}

#[test]
fn ks_zero_for_identical_distributions() {
    let d = uniform_dist(10, 1000);
    let p = d.to_probs();
    let val = ks_statistic(&p, &p).unwrap();
    assert!(val.abs() < 1e-6);
}

#[test]
fn ks_positive_for_gaussian_vs_uniform() {
    let base = uniform_dist(20, 2000);
    let curr = gaussian_approx(20, 2000);
    let p = base.to_probs();
    let q = curr.to_probs();
    let val = ks_statistic(&p, &q).unwrap();
    assert!(val > 0.01, "KS should detect gaussian vs uniform: {val}");
}

#[test]
fn js_divergence_zero_for_identical() {
    let d = uniform_dist(10, 1000);
    let p = d.to_probs();
    let val = js_divergence(&p, &p).unwrap();
    assert!(val.abs() < 1e-6);
}

#[test]
fn js_divergence_bounded() {
    let base = uniform_dist(10, 1000);
    let curr = gaussian_approx(10, 1000);
    let p = base.to_probs();
    let q = curr.to_probs();
    let val = js_divergence(&p, &q).unwrap();
    assert!(
        (0.0..=0.7).contains(&val),
        "JS divergence out of bounds: {val}"
    );
}

#[test]
fn wasserstein_zero_for_identical() {
    let d = uniform_dist(10, 1000);
    let p = d.to_probs();
    let val = wasserstein_distance(&p, &p).unwrap();
    assert!(val.abs() < 1e-6);
}

#[test]
fn psi_errors_on_bin_mismatch() {
    let p = vec![0.5, 0.5];
    let q = vec![0.33, 0.33, 0.34];
    assert!(psi(&p, &q).is_err());
}

#[test]
fn detector_classify_stable_as_info() {
    let detector = DriftDetector::default();
    let base = uniform_dist(10, 1000);
    let curr = uniform_dist(10, 1000);
    let report = detector.detect(&base, &curr, DriftWindow::H24).unwrap();
    assert_eq!(report.severity, DriftSeverity::Info);
}

#[test]
fn detector_classify_shift_as_warning_or_critical() {
    let detector = DriftDetector::default();
    let base = uniform_dist(10, 1000);
    let curr = gaussian_approx(10, 1000);
    let report = detector.detect(&base, &curr, DriftWindow::D7).unwrap();
    assert!(
        report.severity >= DriftSeverity::Warning,
        "expected Warning or Critical"
    );
}

#[test]
fn alert_none_for_info_severity() {
    let detector = DriftDetector::default();
    let base = uniform_dist(10, 1000);
    let curr = uniform_dist(10, 1000);
    let report = detector.detect(&base, &curr, DriftWindow::H24).unwrap();
    assert!(maybe_alert(Uuid::now_v7(), &report).is_none());
}

#[test]
fn alert_some_for_critical_severity() {
    let report = DriftReport {
        feature_name: "f".to_string(),
        window: DriftWindow::D30,
        psi: 0.5,
        ks_stat: 0.3,
        js_div: 0.2,
        wasserstein: 0.1,
        severity: DriftSeverity::Critical,
        computed_at: Utc::now(),
    };
    let alert = maybe_alert(Uuid::now_v7(), &report);
    assert!(alert.is_some());
}

#[test]
fn adaptive_suggestion_needs_7_reports() {
    let make = |psi_v: f64| DriftReport {
        feature_name: "f".to_string(),
        window: DriftWindow::H24,
        psi: psi_v,
        ks_stat: 0.0,
        js_div: 0.0,
        wasserstein: 0.0,
        severity: DriftSeverity::Warning,
        computed_at: Utc::now(),
    };
    let reports: Vec<_> = (0..6).map(|i| make(i as f64 * 0.05)).collect();
    assert!(suggest_adaptive_threshold(&reports).is_none());
    let reports2: Vec<_> = (0..7).map(|i| make(i as f64 * 0.05 + 0.01)).collect();
    let suggestion = suggest_adaptive_threshold(&reports2);
    assert!(suggestion.is_some());
    assert!(suggestion.unwrap() > 0.0);
}
