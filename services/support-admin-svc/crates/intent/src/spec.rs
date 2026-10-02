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

//! Typed models for `support-intent/preservation.yaml`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mirrors `PreMarkValidationRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreMarkValidationRequest {
    pub station_code: String,
    pub contest_id: String,
    pub inference_outcome: String,
    pub model_version: String,
    pub device_id: String,
    pub image_delete_elapsed_ms: i32,
}

/// Mirrors `SpoiltBallotRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoiltBallotRequest {
    pub station_code: String,
    pub contest_id: String,
    pub reason_code: String,
    pub officer_id: String,
    pub witness_ids: Vec<String>,
    pub replacement_issued: Option<bool>,
}

/// Mirrors `AdjudicationRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjudicationRequest {
    pub ballot_id: String,
    pub station_code: String,
    pub contest_id: String,
    pub adjudicator_id: String,
    pub reason_code: String,
    pub decision: String,
    pub photo_evidence_ref: Option<String>,
}

/// Mirrors `AdjudicationRecord`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjudicationRecord {
    pub id: Uuid,
    pub ballot_id: String,
    pub station_code: String,
    pub contest_id: String,
    pub adjudicator_id: String,
    pub reason_code: String,
    pub decision: String,
    pub photo_evidence_ref: Option<String>,
    pub adjudicated_at: DateTime<Utc>,
}

/// Mirrors `RejectionRateAlert`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionRateAlert {
    pub id: Uuid,
    pub station_code: String,
    pub contest_id: String,
    pub rejection_rate: f64,
    pub national_mean: f64,
    pub z_score: f64,
    pub alerted_at: DateTime<Utc>,
    pub acknowledged: bool,
}
