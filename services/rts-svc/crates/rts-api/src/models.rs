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
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CandidateInput {
    pub candidate_id: String,
    pub candidate_name: String,
    pub votes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Form34ARequest {
    pub station_code: String,
    pub constituency_code: String,
    pub county_code: String,
    pub registered_voters: u32,
    pub votes_cast: u32,
    pub valid_votes: u32,
    pub rejected_votes: u32,
    pub spoilt_ballots: u32,
    pub candidates: Vec<CandidateInput>,
    pub raw_scan_ref: Option<String>,
    pub device_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FormAccepted {
    pub form_id: Uuid,
    pub status: String,
    pub icr_job_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SmsFallbackRequest {
    pub encoded_payload: String,
    pub device_id: String,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenRequest {
    pub device_id: String,
    pub svid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AckResponse {
    pub scan_id: Uuid,
    pub status: String,
    pub form_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SnapshotResponse {
    pub scope: String,
    pub code: String,
    pub snapshot_id: Option<Uuid>,
    pub published_at: Option<String>,
    pub form34c: Option<serde_json::Value>,
    pub merkle_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IcrField {
    pub field_name: String,
    pub value: String,
    pub confidence: f64,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IcrJobResult {
    pub job_id: String,
    pub form_id: Uuid,
    pub fields: Vec<IcrField>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProofResponse {
    pub form_id: Uuid,
    pub merkle_proof: Vec<String>,
    pub merkle_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisputeRequest {
    pub form_id: Uuid,
    pub reason: String,
    pub ro_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisputeCreated {
    pub dispute_id: Uuid,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisputeResolveRequest {
    pub resolution: String,
    pub countersign_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisputeResolved {
    pub dispute_id: Uuid,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventIngestRequest {
    pub envelope: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventIngestResponse {
    pub event_id: String,
    pub status: String,
}
