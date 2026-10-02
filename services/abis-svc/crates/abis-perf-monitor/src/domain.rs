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

//! Domain types for performance monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldoutPair {
    pub probe_id: Uuid,
    pub gallery_id: Uuid,
    pub score: f64,
    pub genuine: bool,
    pub rank: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollupPeriod {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub id: Uuid,
    pub model_id: Uuid,
    pub period: RollupPeriod,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub fmr: f64,
    pub fnmr: f64,
    pub rank1_accuracy: f64,
    pub rank5_accuracy: f64,
    pub n_genuine: usize,
    pub n_impostor: usize,
    pub threshold_used: f64,
    pub generated_at: DateTime<Utc>,
}

impl PerformanceReport {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model_id: Uuid,
        period: RollupPeriod,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        fmr: f64,
        fnmr: f64,
        rank1: f64,
        rank5: f64,
        n_genuine: usize,
        n_impostor: usize,
        threshold: f64,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            model_id,
            period,
            period_start,
            period_end,
            fmr,
            fnmr,
            rank1_accuracy: rank1,
            rank5_accuracy: rank5,
            n_genuine,
            n_impostor,
            threshold_used: threshold,
            generated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTarget {
    pub fmr_target: f64,
    pub fnmr_target: f64,
    pub rank1_target: f64,
}

impl Default for PerformanceTarget {
    fn default() -> Self {
        Self {
            fmr_target: 0.00001,
            fnmr_target: 0.02,
            rank1_target: 0.99,
        }
    }
}
