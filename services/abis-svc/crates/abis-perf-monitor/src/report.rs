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
