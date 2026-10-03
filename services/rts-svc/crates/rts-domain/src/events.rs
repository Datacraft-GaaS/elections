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

use crate::aggregates::form34a::CandidateResult;
use serde::{Deserialize, Serialize};

pub const EVENT_FORM34A_CAPTURED: &str = "eel.rts.form34a_captured.v1";
pub const EVENT_FORM34A_VERIFIED: &str = "eel.rts.form34a_verified.v1";
pub const EVENT_FORM34A_AMENDED: &str = "eel.rts.form34a_amended.v1";
pub const EVENT_FORM34A_REJECTED: &str = "eel.rts.form34a_rejected.v1";
pub const EVENT_FORM34B_COMPILED: &str = "eel.rts.form34b_compiled.v1";
pub const EVENT_FORM34C_COMPILED: &str = "eel.rts.form34c_compiled.v1";
pub const EVENT_TALLY_PUBLISHED: &str = "eel.rts.tally_published.v1";
pub const EVENT_PARALLEL_TALLY_DRIFT: &str = "eel.rts.parallel_tally_drift.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34ACapturedPayload {
    pub form_id: String,
    pub station_code: String,
    pub constituency_code: String,
    pub county_code: String,
    pub registered_voters: u32,
    pub votes_cast: u32,
    pub valid_votes: u32,
    pub rejected_votes: u32,
    pub spoilt_ballots: u32,
    pub candidates: Vec<CandidateResult>,
    pub raw_scan_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34AVerifiedPayload {
    pub form_id: String,
    pub verified_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34AAmendedPayload {
    pub form_id: String,
    pub valid_votes: u32,
    pub rejected_votes: u32,
    pub spoilt_ballots: u32,
    pub votes_cast: u32,
    pub candidates: Vec<CandidateResult>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34ARejectedPayload {
    pub form_id: String,
    pub reason: String,
    pub rejected_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34BCompiledPayload {
    pub form_id: String,
    pub constituency_code: String,
    pub county_code: String,
    pub compiled_stations: Vec<String>,
    pub total_registered: u32,
    pub total_valid: u32,
    pub total_rejected: u32,
    pub total_spoilt: u32,
    pub candidate_totals: Vec<CandidateResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form34CCompiledPayload {
    pub form_id: String,
    pub compiled_constituencies: Vec<String>,
    pub total_registered: u64,
    pub total_valid: u64,
    pub total_rejected: u64,
    pub total_spoilt: u64,
    pub candidate_totals: Vec<CandidateResult>,
    pub merkle_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TallyPublishedPayload {
    pub snapshot_id: String,
    pub merkle_root: String,
    pub signed_url: String,
    pub tlog_leaf_index: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelTallyDriftPayload {
    pub aggregator_id: String,
    pub candidate_id: String,
    pub expected: u64,
    pub actual: u64,
}
