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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportTicket {
    pub id: Uuid,
    pub station_code: String,
    pub kit_serial: String,
    pub reporter_role: String,
    pub severity: TicketSeverity,
    pub title: String,
    pub description: String,
    pub status: TicketStatus,
    pub assigned_engineer_id: Option<Uuid>,
    pub sla_target_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub escalation_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TicketSeverity {
    Sev1,
    Sev2,
    Sev3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TicketStatus {
    Open,
    InProgress,
    Escalated,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticAction {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub kit_serial: String,
    pub action_type: DiagnosticActionType,
    pub engineer_id: Uuid,
    pub kit_consent_recorded: bool,
    pub signed_at: DateTime<Utc>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticActionType {
    FetchLogs,
    Reboot,
    FirmwareAttest,
    SelfTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarrantyClaim {
    pub id: Uuid,
    pub kit_serial: String,
    pub oem_code: String,
    pub fault_description: String,
    pub claim_status: String,
    pub engineer_id: Uuid,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTicketRequest {
    pub station_code: String,
    pub kit_serial: String,
    pub severity: TicketSeverity,
    pub title: String,
    pub description: String,
}
