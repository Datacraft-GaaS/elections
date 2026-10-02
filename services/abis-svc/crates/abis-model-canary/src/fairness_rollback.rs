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

//! HARD-23: auto-rollback on fairness-slice breach.
//!
//! Extends the existing canary monitor loop to also trigger rollback when any
//! demographic slice's error rate deviates beyond a configured tolerance from
//! the baseline model's rate on the same slice.
//!
//! Data source: `eel_fairness_evaluations` table (V024) produced by
//! `abis-perf-monitor`.  This module reads rows from that table and computes
//! the maximum absolute deviation across all slices.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::domain::RollbackReason;

/// A single fairness evaluation row from `eel_fairness_evaluations` (V024).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessEvaluation {
    /// The demographic slice label (e.g. "age_65_plus", "region_coast").
    pub slice: String,
    /// FMR on this slice for the canary model.
    pub canary_fmr: f64,
    /// FMR on this slice for the baseline model.
    pub baseline_fmr: f64,
    /// FNMR on this slice for the canary model.
    pub canary_fnmr: f64,
    /// FNMR on this slice for the baseline model.
    pub baseline_fnmr: f64,
}

/// Configuration for the fairness breach detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessRollbackConfig {
    /// Maximum allowed absolute deviation (in percentage points) between
    /// canary and baseline FMR on any single slice before a rollback is triggered.
    pub fmr_tolerance: f64,
    /// Maximum allowed absolute deviation for FNMR.
    pub fnmr_tolerance: f64,
}

impl Default for FairnessRollbackConfig {
    fn default() -> Self {
        Self {
            fmr_tolerance: 0.005,  // 0.5 pp
            fnmr_tolerance: 0.010, // 1.0 pp
        }
    }
}

/// Detect whether any fairness slice exceeds the configured tolerances.
///
/// Returns `Some(RollbackReason::FairnessSliceBreach {...})` if a breach is
/// detected, `None` if all slices are within tolerance.
pub fn detect_fairness_breach(
    evals: &[FairnessEvaluation],
    config: &FairnessRollbackConfig,
) -> Option<RollbackReason> {
    for eval in evals {
        let fmr_dev = (eval.canary_fmr - eval.baseline_fmr).abs();
        if fmr_dev > config.fmr_tolerance {
            warn!(
                slice = %eval.slice,
                deviation = fmr_dev,
                tolerance = config.fmr_tolerance,
                "FMR fairness breach on slice"
            );
            return Some(RollbackReason::FairnessSliceBreach {
                slice: eval.slice.clone(),
                metric: "fmr".to_owned(),
                deviation: fmr_dev,
                tolerance: config.fmr_tolerance,
            });
        }
        let fnmr_dev = (eval.canary_fnmr - eval.baseline_fnmr).abs();
        if fnmr_dev > config.fnmr_tolerance {
            warn!(
                slice = %eval.slice,
                deviation = fnmr_dev,
                tolerance = config.fnmr_tolerance,
                "FNMR fairness breach on slice"
            );
            return Some(RollbackReason::FairnessSliceBreach {
                slice: eval.slice.clone(),
                metric: "fnmr".to_owned(),
                deviation: fnmr_dev,
                tolerance: config.fnmr_tolerance,
            });
        }
    }
    None
}

/// Called from the canary monitor loop.  If a fairness breach is detected,
/// applies the rollback to the mutable `CanaryRecord` and returns the reason.
///
/// Callers should persist the updated record and emit a `RollbackEvent`
/// afterwards (same pattern as `monitor::check_sla` + `rollback::rollback`).
pub fn rollback_on_fairness_breach(
    record: &mut crate::domain::CanaryRecord,
    evals: &[FairnessEvaluation],
    config: &FairnessRollbackConfig,
) -> Option<RollbackReason> {
    let reason = detect_fairness_breach(evals, config)?;
    // Idempotent: if already rolled back, skip.
    if record.state != crate::domain::CanaryState::InProgress {
        return None;
    }
    let _ = crate::rollback::rollback(record, reason.clone());
    Some(reason)
}
