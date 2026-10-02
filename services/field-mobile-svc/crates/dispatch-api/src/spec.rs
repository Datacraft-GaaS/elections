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

//! Public contract models (field-mobile-dispatch/dispatch.yaml).

use serde::{Deserialize, Serialize};

/// Request body for creating a dispatch assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentCreate {
    pub personnel_id_hash: String,
    pub station_code: String,
    pub role: String,
    pub report_by: String,
    pub election_day: String,
    pub notes: String,
}

/// Public dispatch assignment view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchAssignment {
    pub assignment_id: String,
    pub personnel_id_hash: String,
    pub station_code: String,
    pub role: String,
    pub report_by: String,
    pub election_day: String,
    pub status: String,
    pub actual_arrival: Option<String>,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for patching a dispatch assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentPatch {
    pub station_code: Option<String>,
    pub personnel_id_hash: Option<String>,
    pub report_by: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

/// Request body for a bulk assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAssignRequest {
    pub upload_ref: String,
    pub dry_run: bool,
    pub election_day: String,
}

/// Public bulk-assignment job view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAssignJob {
    pub job_id: String,
    pub status: String,
    pub dry_run: bool,
    pub submitted_at: String,
    pub created_count: u32,
    pub error_count: u32,
    pub error_report_ref: Option<String>,
}

/// One station's coverage row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationCoverage {
    pub station_code: String,
    pub required_count: u32,
    pub assigned_count: u32,
    pub coverage_status: String,
}

/// Public constituency coverage view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstituencyCoverage {
    pub constituency_code: String,
    pub election_day: String,
    pub total_stations: u32,
    pub fully_staffed_count: u32,
    pub understaffed_count: u32,
    pub unstaffed_count: u32,
    pub coverage_pct: f64,
    pub stations: Vec<StationCoverage>,
}
