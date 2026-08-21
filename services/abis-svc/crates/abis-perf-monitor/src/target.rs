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
