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

//! Tests for fairness-slice auto-rollback (HARD-23). 12+ cases.

use eel_abis_model_canary::{
    canary::start_canary,
    domain::{CanaryState, RollbackReason},
    fairness_rollback::{
        detect_fairness_breach, rollback_on_fairness_breach, FairnessEvaluation,
        FairnessRollbackConfig,
    },
};
use uuid::Uuid;

fn default_config() -> FairnessRollbackConfig {
    FairnessRollbackConfig {
        fmr_tolerance: 0.005,
        fnmr_tolerance: 0.010,
    }
}

fn baseline_eval(slice: &str) -> FairnessEvaluation {
    FairnessEvaluation {
        slice: slice.to_owned(),
        canary_fmr: 0.010,
        baseline_fmr: 0.010,
        canary_fnmr: 0.050,
        baseline_fnmr: 0.050,
    }
}

fn default_record() -> eel_abis_model_canary::domain::CanaryRecord {
    start_canary(Uuid::now_v7(), Uuid::now_v7(), 10, 0.01, 0.05, 500.0, 0.01).unwrap()
}

// ---- detector fires on breach ----

#[test]
fn fmr_breach_detected() {
    let mut eval = baseline_eval("age_65_plus");
    eval.canary_fmr = 0.020; // +1.0 pp deviation, tolerance = 0.5 pp
    let result = detect_fairness_breach(&[eval], &default_config());
    assert!(result.is_some());
    assert!(matches!(
        result.unwrap(),
        RollbackReason::FairnessSliceBreach { metric, .. } if metric == "fmr"
    ));
}

#[test]
fn fnmr_breach_detected() {
    let mut eval = baseline_eval("region_coast");
    eval.canary_fnmr = 0.065; // +1.5 pp deviation, tolerance = 1.0 pp
    let result = detect_fairness_breach(&[eval], &default_config());
    assert!(result.is_some());
    assert!(matches!(
        result.unwrap(),
        RollbackReason::FairnessSliceBreach { metric, .. } if metric == "fnmr"
    ));
}

#[test]
fn breach_reports_correct_slice() {
    let mut eval = baseline_eval("gender_female");
    eval.canary_fmr = 0.020;
    let reason = detect_fairness_breach(&[eval], &default_config()).unwrap();
    match reason {
        RollbackReason::FairnessSliceBreach { slice, .. } => {
            assert_eq!(slice, "gender_female");
        }
        _ => panic!("wrong reason variant"),
    }
}

// ---- no false rollback on baseline ----

#[test]
fn no_breach_when_within_tolerance() {
    let eval = baseline_eval("age_18_24");
    assert!(detect_fairness_breach(&[eval], &default_config()).is_none());
}

#[test]
fn no_breach_on_empty_evaluations() {
    assert!(detect_fairness_breach(&[], &default_config()).is_none());
}

#[test]
fn deviation_exactly_at_tolerance_boundary_is_ok() {
    let mut eval = baseline_eval("region_north");
    // Exactly at tolerance -> no breach
    eval.canary_fmr = eval.baseline_fmr + 0.005;
    assert!(detect_fairness_breach(&[eval], &default_config()).is_none());
}

// ---- tolerance configurable ----

#[test]
fn custom_strict_tolerance_triggers_on_small_deviation() {
    let strict = FairnessRollbackConfig {
        fmr_tolerance: 0.001, // 0.1 pp
        fnmr_tolerance: 0.001,
    };
    let mut eval = baseline_eval("age_65_plus");
    eval.canary_fmr = 0.012; // +0.2 pp
    assert!(detect_fairness_breach(&[eval], &strict).is_some());
}

#[test]
fn custom_lenient_tolerance_does_not_trigger() {
    let lenient = FairnessRollbackConfig {
        fmr_tolerance: 0.050, // 5.0 pp
        fnmr_tolerance: 0.050,
    };
    let mut eval = baseline_eval("age_65_plus");
    eval.canary_fmr = 0.030; // +2.0 pp, within 5 pp tolerance
    assert!(detect_fairness_breach(&[eval], &lenient).is_none());
}

// ---- rollback issued ----

#[test]
fn rollback_applied_on_breach() {
    let mut record = default_record();
    let mut eval = baseline_eval("age_65_plus");
    eval.canary_fmr = 0.020;
    let reason = rollback_on_fairness_breach(&mut record, &[eval], &default_config());
    assert!(reason.is_some());
    assert_eq!(record.state, CanaryState::RolledBack);
    assert_eq!(record.traffic_pct, 0);
}

#[test]
fn rollback_not_applied_when_no_breach() {
    let mut record = default_record();
    let eval = baseline_eval("age_65_plus");
    let reason = rollback_on_fairness_breach(&mut record, &[eval], &default_config());
    assert!(reason.is_none());
    assert_eq!(record.state, CanaryState::InProgress);
}

#[test]
fn rollback_idempotent_if_already_rolled_back() {
    let mut record = default_record();
    let mut eval = baseline_eval("age_65_plus");
    eval.canary_fmr = 0.020;
    // First rollback
    rollback_on_fairness_breach(&mut record, &[eval.clone()], &default_config());
    assert_eq!(record.state, CanaryState::RolledBack);
    // Second call -> should not error or change state
    let reason2 = rollback_on_fairness_breach(&mut record, &[eval], &default_config());
    assert!(reason2.is_none());
    assert_eq!(record.state, CanaryState::RolledBack);
}

// ---- event emitted (rollback_reason field set) ----

#[test]
fn rollback_reason_field_set_on_fairness_breach() {
    let mut record = default_record();
    let mut eval = baseline_eval("age_65_plus");
    eval.canary_fmr = 0.020;
    rollback_on_fairness_breach(&mut record, &[eval], &default_config());
    assert!(record.rollback_reason.is_some());
    let reason_str = record.rollback_reason.unwrap();
    assert!(reason_str.contains("FairnessSliceBreach") || reason_str.contains("fairness"));
}

#[test]
fn multiple_slice_evals_first_breach_wins() {
    let ok = baseline_eval("region_coast");
    let mut bad = baseline_eval("age_65_plus");
    bad.canary_fmr = 0.020;
    let result = detect_fairness_breach(&[ok, bad], &default_config());
    assert!(result.is_some());
}
