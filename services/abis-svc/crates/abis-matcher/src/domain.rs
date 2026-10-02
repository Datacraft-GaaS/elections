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

//! Core domain types for the ABIS matching pipeline.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateIndex {
    pub index_id: String,
    pub voter_id: String,
    pub modality: String,
    pub template_hash: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchRequest {
    pub request_id: String,
    pub probe_ref: String,
    pub modality: String,
    pub election_id: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchResult {
    pub result_id: String,
    pub request_id: String,
    pub outcome: MatchOutcome,
    pub confidence_score: f32,
    pub modality_scores: Vec<ModalityScore>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchScore {
    pub score: f32,
    pub threshold: f32,
    pub outcome: MatchOutcome,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModalityScore {
    pub modality: String,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchOutcome {
    Match,
    NoMatch,
    Adjudicate,
    InsufficientQuality,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdjudicationCase {
    pub case_id: String,
    pub match_result_id: String,
    pub voter_ids: Vec<String>,
    pub status: AdjudicationStatus,
    pub investigator_id: Option<String>,
    pub decision: Option<String>,
    pub reason_code: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjudicationStatus {
    Open,
    InReview,
    Resolved,
    Escalated,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdjudicationDecision {
    pub case_id: String,
    pub decision: String,
    pub reason_code: String,
    pub officer_id: String,
    pub decided_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnclaveAttestation {
    pub enclave_id: String,
    pub worker_id: String,
    pub verified_at: String,
    pub profile: AttestationProfile,
    pub attestation_hash: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationProfile {
    Software,
    Tdx,
    Sevsnp,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriftAlert {
    pub alert_id: String,
    pub detected_at: String,
    pub drift_type: String,
    pub value: f32,
    pub threshold: f32,
}
