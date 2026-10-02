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

//! Public contract models (governance-rla/audit.yaml).

use serde::{Deserialize, Serialize};

/// Request body for initiating a risk-limiting audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlaAuditInitRequest {
    pub election_id: String,
    pub contest_code: String,
    pub risk_limit: f64,
    pub method: String,
    pub seed: String,
}

/// Public RLA audit view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlaAudit {
    pub audit_id: String,
    pub election_id: String,
    pub contest_code: String,
    pub risk_limit: f64,
    pub method: String,
    pub seed: String,
    pub status: String,
    pub current_round: u32,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for drawing a sample round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleDrawRequest {
    pub round: u32,
}

/// One drawn sample ballot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleBallot {
    pub ballot_id_hash: String,
    pub station_code: String,
    pub form_id: String,
}

/// Public sample list view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleList {
    pub audit_id: String,
    pub round: u32,
    pub samples: Vec<SampleBallot>,
}

/// Request body for submitting a manual count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualCount {
    pub ballot_id_hash: String,
    pub cvr_candidate_id: String,
    pub audit_candidate_id: String,
    pub matches: bool,
    pub counted_by_hash: String,
    pub counted_at: Option<String>,
}

/// Public RLA results view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlaAuditResults {
    pub audit_id: String,
    pub p_value: f64,
    pub risk_limit: f64,
    pub risk_limit_met: bool,
    pub stopping_condition_met: bool,
    pub ballots_audited: u32,
    pub discrepancies: u32,
    pub computed_at: String,
}
