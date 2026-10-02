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

//! Typed models for `printing/chain.yaml`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mirrors `PrintOrderRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintOrderRequest {
    /// election_id.
    pub election_id: Uuid,
    /// position_code.
    pub position_code: String,
    /// constituency_code.
    pub constituency_code: String,
    /// ballot_quantity.
    pub ballot_quantity: i32,
    /// proof_id.
    pub proof_id: Uuid,
    /// security_features.
    pub security_features: Vec<String>,
}

/// Mirrors `PrintOrder`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintOrder {
    /// order_id.
    pub order_id: Uuid,
    /// election_id.
    pub election_id: Uuid,
    /// position_code.
    pub position_code: String,
    /// constituency_code.
    pub constituency_code: String,
    /// ballot_quantity.
    pub ballot_quantity: i32,
    /// proof_id.
    pub proof_id: Uuid,
    /// security_features.
    pub security_features: Vec<String>,
    /// status.
    pub status: String,
    /// printer_reference.
    pub printer_reference: String,
    /// created_at.
    pub created_at: DateTime<Utc>,
    /// updated_at.
    pub updated_at: DateTime<Utc>,
}

/// Mirrors `QCDecision`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QCDecision {
    /// passed.
    pub passed: bool,
    /// defect_count.
    pub defect_count: i32,
    /// sample_size.
    pub sample_size: i32,
    /// qc_officer_hash.
    pub qc_officer_hash: String,
    /// decided_at.
    pub decided_at: DateTime<Utc>,
    /// notes.
    pub notes: Option<String>,
}

/// Mirrors `DeliveryRecordRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryRecordRequest {
    /// order_id.
    pub order_id: Uuid,
    /// delivered_quantity.
    pub delivered_quantity: i32,
    /// serial_range_start.
    pub serial_range_start: String,
    /// serial_range_end.
    pub serial_range_end: String,
    /// received_by_hash.
    pub received_by_hash: String,
    /// received_at.
    pub received_at: DateTime<Utc>,
}

/// Mirrors `DeliveryRecord`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryRecord {
    /// delivery_id.
    pub delivery_id: Uuid,
    /// order_id.
    pub order_id: Uuid,
    /// delivered_quantity.
    pub delivered_quantity: i32,
    /// serial_range_start.
    pub serial_range_start: String,
    /// serial_range_end.
    pub serial_range_end: String,
    /// received_by_hash.
    pub received_by_hash: String,
    /// received_at.
    pub received_at: DateTime<Utc>,
    /// status.
    pub status: String,
    /// created_at.
    pub created_at: DateTime<Utc>,
}

/// Mirrors `AuditEvent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// event_id.
    pub event_id: Uuid,
    /// event_type.
    pub event_type: String,
    /// order_id.
    pub order_id: Option<Uuid>,
    /// actor_role.
    pub actor_role: Option<String>,
    /// payload_summary.
    pub payload_summary: serde_json::Value,
    /// created_at.
    pub created_at: DateTime<Utc>,
}
