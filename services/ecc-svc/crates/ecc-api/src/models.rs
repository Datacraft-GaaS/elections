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
use uuid::Uuid;

// ECC models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SituationalAwareness {
    pub total_stations: u32,
    pub reporting_stations: u32,
    pub pending_escalations: u32,
    pub active_alerts: u32,
    pub turnout_pct: f64,
    pub as_of: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub station_code: String,
    pub name: String,
    pub constituency_code: String,
    pub county_code: String,
    pub status: String,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escalation {
    pub escalation_id: Uuid,
    pub station_code: String,
    pub category: String,
    pub severity: String,
    pub status: String,
    pub description: String,
    pub raised_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRequest {
    pub station_code: String,
    pub category: String,
    pub severity: String,
    pub description: String,
    pub raised_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationCreated {
    pub escalation_id: Uuid,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPatch {
    pub status: Option<String>,
    pub resolution_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnoutHeatmapEntry {
    pub constituency_code: String,
    pub county_code: String,
    pub turnout_pct: f64,
    pub votes_cast: u32,
    pub registered_voters: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub entry_id: Uuid,
    pub action: String,
    pub actor: String,
    pub resource: String,
    pub timestamp: String,
}

// IAB models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_id: Uuid,
    pub module_code: String,
    pub title: String,
    pub status: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReportRequest {
    pub module_code: String,
    pub title: String,
    pub body: String,
    pub submitted_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReportCreated {
    pub report_id: Uuid,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IabSummary {
    pub open_reports: u32,
    pub flagged_items: u32,
    pub pending_trails: u32,
    pub as_of: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEntry {
    pub trail_id: Uuid,
    pub module_code: String,
    pub event: String,
    pub actor: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IabFlag {
    pub flag_id: Uuid,
    pub module_code: String,
    pub description: String,
    pub severity: String,
    pub raised_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IabFlagRequest {
    pub module_code: String,
    pub description: String,
    pub severity: String,
    pub raised_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IabFlagCreated {
    pub flag_id: Uuid,
    pub status: String,
}

// Grievance models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrievanceComplaint {
    pub complaint_id: Uuid,
    pub reference_code: String,
    pub complainant_name: String,
    pub category: String,
    pub status: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrievanceComplaintRequest {
    pub complainant_name: String,
    pub complainant_contact: String,
    pub category: String,
    pub description: String,
    pub station_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrievanceComplaintCreated {
    pub complaint_id: Uuid,
    pub reference_code: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplaintUpdate {
    pub update_id: Uuid,
    pub complaint_id: Uuid,
    pub note: String,
    pub updated_by: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplaintUpdateRequest {
    pub note: String,
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UssdSessionRequest {
    pub session_id: String,
    pub msisdn: String,
    pub input: String,
    pub network_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UssdSessionResponse {
    pub session_id: String,
    pub response: String,
    pub end_session: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsappWebhookPayload {
    pub object: String,
    pub entry: serde_json::Value,
}

// Section 44A dual-officer verification

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S44aOfficer1Request {
    pub county_code: String,
    pub station_code: String,
    pub officer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S44aOfficer2Request {
    pub station_code: String,
    pub officer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S44aConfirmation {
    pub confirmation_id: Uuid,
    pub station_code: String,
    pub status: String,
}
