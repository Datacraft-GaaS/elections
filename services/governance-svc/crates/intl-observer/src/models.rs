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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MissionStatus {
    Registered,
    Active,
    Closed,
}

impl Default for MissionStatus {
    fn default() -> Self {
        Self::Registered
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    Interim,
    Final,
}

impl MissionStatus {
    /// The wire value, derived from the serde representation so the two cannot
    /// drift apart.
    pub fn wire(&self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
    }
}

impl Default for ReportType {
    fn default() -> Self {
        Self::Interim
    }
}

/// A registered observer mission.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObserverMission {
    pub id: String,
    pub organisation_name: String,
    pub accreditation_ref: String,
    pub lead_contact_hash: String, // hashed - class B PII
    pub workspace_id: String,
    pub status: MissionStatus,
    pub registered_at: String,
}

/// A briefing scheduled for a mission.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Briefing {
    pub id: String,
    pub mission_id: String,
    pub topic: String,
    pub scheduled_at: String,
    pub materials_url: Option<String>,
    pub attendance_confirmed: bool,
}

/// An observation report filed by a mission.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObservationReport {
    pub id: String,
    pub mission_id: String,
    pub report_type: ReportType,
    pub submitted_at: String,
    pub content_url: String,
    pub classified: bool,
}

/// A joint statement co-published by missions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JointStatement {
    pub id: String,
    pub participating_mission_ids: Vec<String>,
    pub published_at: String,
    pub content_url: String,
    pub signed_by: Vec<String>,
}

/// A registration request.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegisterMissionRequest {
    pub organisation_name: String,
    pub accreditation_ref: String,
    pub lead_contact_hash: String,
}

/// A briefing scheduling request.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleBriefingRequest {
    pub topic: String,
    pub scheduled_at: String,
    pub materials_url: Option<String>,
}

/// An observation report submission.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitReportRequest {
    pub report_type: ReportType,
    pub content_url: String,
    pub classified: bool,
}

/// A joint statement publication request.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublishJointStatementRequest {
    pub participating_mission_ids: Vec<String>,
    pub content_url: String,
    pub signed_by: Vec<String>,
}
