//! FRVT report card.

use crate::metrics::{FairnessSlice, FmrFnmr, QualityCorrelation, RankN};
use serde::{Deserialize, Serialize};

/// Full FRVT evaluation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrvtReport {
    pub verification: FmrFnmr,
    pub rank1: RankN,
    pub quality_correlation: QualityCorrelation,
    pub fairness_slices: Vec<FairnessSlice>,
}

impl FrvtReport {
    pub fn new(
        verification: FmrFnmr,
        rank1: RankN,
        quality_correlation: QualityCorrelation,
        fairness_slices: Vec<FairnessSlice>,
    ) -> Self {
        Self {
            verification,
            rank1,
            quality_correlation,
            fairness_slices,
        }
    }

    /// Summary string for logging.
    pub fn summary(&self) -> String {
        format!(
            "FRVT: FMR={:.4} FNMR={:.4} Rank1={:.4} QualityR={:.4} Slices={}",
            self.verification.fmr,
            self.verification.fnmr,
            self.rank1.hit_rate,
            self.quality_correlation.pearson_r,
            self.fairness_slices.len(),
        )
    }
}
