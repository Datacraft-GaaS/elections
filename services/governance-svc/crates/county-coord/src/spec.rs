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

//! Public contract models (governance-county-coord/coordination.yaml).

use serde::{Deserialize, Serialize};

/// Request body for submitting a county status report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyStatusReportRequest {
    pub county_code: String,
    pub stations_open: u32,
    pub stations_closed_early: u32,
    pub incidents: u32,
    pub notes: Option<String>,
    pub submitted_at: String,
}

/// Public county status report view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyStatusReport {
    pub report_id: String,
    pub county_code: String,
    pub stations_open: u32,
    pub stations_closed_early: u32,
    pub incidents: u32,
    pub notes: Option<String>,
    pub submitted_at: String,
    pub created_at: String,
}

/// Request body for escalating an incident to the national level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationEscalationRequest {
    pub county_code: String,
    pub incident_type: String,
    pub severity: String,
    pub description: String,
}

/// Public escalation view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationEscalation {
    pub escalation_id: String,
    pub county_code: String,
    pub incident_type: String,
    pub severity: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
