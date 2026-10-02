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
pub struct RollSnapshot {
    pub id: Uuid,
    /// BLAKE3 hash over sorted voter IDs, hex-encoded.
    pub snapshot_hash: String,
    pub taken_at: DateTime<Utc>,
    pub station_count: u32,
    pub voter_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ObjectionStatus {
    Filed,
    UnderReview,
    Sustained,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objection {
    pub id: Uuid,
    /// BLAKE3 hash of voter ID — never store raw.
    pub voter_id_hash: String,
    pub station_id: String,
    pub reason: String,
    pub filed_by: Uuid,
    pub filed_at: DateTime<Utc>,
    pub status: ObjectionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionType {
    NameSpelling,
    AddressUpdate,
    StationTransfer,
    DeduplicationRemoval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionStatus {
    Pending,
    Approved,
    Rejected,
    Referred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionClaim {
    pub id: Uuid,
    /// BLAKE3 hash of voter ID — never store raw.
    pub voter_id_hash: String,
    /// BLAKE3 hash of biometric template.
    pub claimed_by_biometric_hash: String,
    pub correction_type: CorrectionType,
    pub evidence_refs: Vec<String>,
    pub status: CorrectionStatus,
    pub filed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Determination {
    pub id: Uuid,
    pub claim_id: Uuid,
    pub decision: String,
    pub decided_by: Uuid,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReferralPacket {
    pub id: Uuid,
    pub correction_claim_id: Uuid,
    pub ecourt_ref: String,
    pub referred_at: DateTime<Utc>,
}
