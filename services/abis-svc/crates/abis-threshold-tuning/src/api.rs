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

//! REST API stubs for threshold profiles.

use uuid::Uuid;

use crate::domain::{LabelledPair, SimulateResult, ThresholdProfile};
use crate::error::ThresholdError;
use crate::tuner::ThresholdTuner;

/// Request body for POST /v1/threshold-profiles/simulate.
#[derive(Debug, serde::Deserialize)]
pub struct SimulateRequest {
    pub pairs: Vec<LabelledPair>,
    pub target_fmr: f64,
    pub n_bootstrap: Option<u32>,
}

/// Run a simulate (dry-run) against a labelled sample without persisting anything.
pub fn simulate(req: SimulateRequest) -> Result<SimulateResult, ThresholdError> {
    let tuner = ThresholdTuner::new(req.n_bootstrap.unwrap_or(1000));
    tuner.tune(&req.pairs, req.target_fmr)
}

/// Placeholder type for profile-create request.
#[derive(Debug, serde::Deserialize)]
pub struct CreateProfileRequest {
    pub profile: ThresholdProfile,
}

/// Placeholder type for activate request.
#[derive(Debug, serde::Deserialize)]
pub struct ActivateRequest {
    pub id: Uuid,
}
