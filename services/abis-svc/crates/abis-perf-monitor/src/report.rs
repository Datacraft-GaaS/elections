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

//! Report generation for daily/weekly/monthly rollups.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{PerformanceReport, RollupPeriod};
use crate::pmr::PmrResult;

pub fn build_report(
    model_id: Uuid,
    period: RollupPeriod,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    result: &PmrResult,
    threshold: f64,
) -> PerformanceReport {
    PerformanceReport::new(
        model_id,
        period,
        period_start,
        period_end,
        result.fmr,
        result.fnmr,
        result.rank1_accuracy,
        result.rank5_accuracy,
        result.n_genuine,
        result.n_impostor,
        threshold,
    )
}
