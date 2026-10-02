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
pub struct VoterRegisterEvent {
    pub id: Uuid,
    pub voter_id: Uuid,
    pub event_type: RegisterEventType,
    pub source: EventSource,
    pub confidence_score: f32,
    pub batch_id: Option<Uuid>,
    pub officer_id: Option<Uuid>,
    pub decision: EventDecision,
    pub decision_basis: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RegisterEventType {
    DeathProposed,
    DeathConfirmed,
    IdCorrection,
    DormancyFlag,
    FamilyNotification,
    RemovalApproved,
    RemovalRejected,
    Reinstatement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventSource {
    Crs,
    Nrb,
    Kra,
    FamilyShortcode,
    OfficerManual,
    /// M35 field-officer mobile capture (V050 migration, FIELD_OFFICER source).
    FieldOfficer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventDecision {
    Approved,
    Rejected,
    Pending,
    Held,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterSnapshot {
    pub id: Uuid,
    pub snapshot_at: DateTime<Utc>,
    pub total_registered: i64,
    pub root_hash: String,
    pub merkle_root: String,
    pub gazette_notice_id: Option<Uuid>,
    pub signed_by: Option<Uuid>,
    pub signed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchIngestionReport {
    pub id: Uuid,
    pub source: EventSource,
    pub received_at: DateTime<Utc>,
    pub record_count: u64,
    pub matched: u64,
    pub unmatched: u64,
    pub flagged: u64,
    pub actioned: u64,
    pub errors: u64,
    pub report_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchIngestRequest {
    pub source: EventSource,
    pub records: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyNotificationRequest {
    pub family_shortcode: String,
    pub voter_id_hash: String,
    pub notification_type: RegisterEventType,
}
