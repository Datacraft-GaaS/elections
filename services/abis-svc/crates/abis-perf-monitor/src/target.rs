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

//! Target-vs-actual comparison and alerting.

use tracing::warn;

use crate::domain::PerformanceTarget;
use crate::pmr::PmrResult;

#[derive(Debug, Clone)]
pub struct TargetComparison {
    pub fmr_ok: bool,
    pub fnmr_ok: bool,
    pub rank1_ok: bool,
}

pub fn compare_to_target(result: &PmrResult, target: &PerformanceTarget) -> TargetComparison {
    let fmr_ok = result.fmr <= target.fmr_target;
    let fnmr_ok = result.fnmr <= target.fnmr_target;
    let rank1_ok = result.rank1_accuracy >= target.rank1_target;
    if !fmr_ok {
        warn!(
            fmr = result.fmr,
            target = target.fmr_target,
            "FMR exceeds target"
        );
    }
    if !fnmr_ok {
        warn!(
            fnmr = result.fnmr,
            target = target.fnmr_target,
            "FNMR exceeds target"
        );
    }
    if !rank1_ok {
        warn!(
            rank1 = result.rank1_accuracy,
            target = target.rank1_target,
            "Rank-1 below target"
        );
    }
    TargetComparison {
        fmr_ok,
        fnmr_ok,
        rank1_ok,
    }
}

pub fn all_targets_met(cmp: &TargetComparison) -> bool {
    cmp.fmr_ok && cmp.fnmr_ok && cmp.rank1_ok
}
