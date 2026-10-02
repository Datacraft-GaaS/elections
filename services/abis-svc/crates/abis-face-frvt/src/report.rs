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
