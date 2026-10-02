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

//! API request/response models for the ABIS service.

use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
	Fingerprint,
	Iris,
	Face,
	Multimodal,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchOutcome {
	Match,
	NoMatch,
	Inconclusive,
	Adjudicate,
	InsufficientQuality,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct Match1NRequest {
	pub probe_ref: String,
	pub modality: String,
	pub caller_id: String,
	pub election_id: String,
	#[serde(default)]
	pub threshold_override: Option<f32>,
	#[serde(default)]
	pub max_candidates: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct Match11Request {
	pub probe_ref: String,
	pub gallery_ref: String,
	pub modality: String,
	pub caller_id: String,
	#[serde(default)]
	pub purpose: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct MatchAccepted {
	pub request_id: String,
	pub status: String,
	pub estimated_completion_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct Match1NResult {
	pub request_id: String,
	pub status: String,
	#[serde(default)]
	pub candidates: Vec<MatchCandidate>,
	pub submitted_at: DateTime<Utc>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub completed_at: Option<DateTime<Utc>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct MatchCandidate {
	pub gallery_ref: String,
	pub score: f32,
	pub decision: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub modality: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct Match11Result {
	pub request_id: String,
	pub decision: String,
	pub score: f32,
	pub threshold_used: f32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub modality: Option<String>,
	pub processed_at: DateTime<Utc>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub engine_version: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdjudicationStatus {
	Open,
	InReview,
	Resolved,
	Escalated,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct AdjudicationCase {
	pub case_id: String,
	pub status: String,
	pub match_score: f32,
	pub probe_ref: String,
	pub candidate_gallery_ref: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub outcome: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolution_notes: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub adjudicator_id: Option<String>,
	pub created_at: DateTime<Utc>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct PaginationMeta {
	pub total: i64,
	pub page: u32,
	pub page_size: u32,
	pub has_next: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct AdjudicationCasePage {
	pub cases: Vec<AdjudicationCase>,
	pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct AdjudicationResolutionRequest {
	pub outcome: String,
	pub adjudicator_id: String,
	pub resolution_notes: String,
	#[serde(default)]
	pub supporting_refs: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct BatchDedupRequest {
	pub election_id: String,
	pub mode: String,
	pub requester_id: String,
	#[serde(default)]
	pub modality: Option<String>,
	#[serde(default)]
	pub priority: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct BatchJobAccepted {
	pub job_id: String,
	pub status: String,
	pub estimated_duration_s: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub probe_count: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct AbisReport {
	pub generated_at: DateTime<Utc>,
	pub gallery_size: i64,
	pub pending_dedup_count: i64,
	pub open_adjudication_count: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub match_throughput_per_hour: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub duplicate_rate_pct: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub p50_match_ms: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub p95_match_ms: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub p99_match_ms: Option<f64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub error_rate_pct: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct HealthResponse {
	pub ok: bool,
}

/// Contract query parameters for GET /api/v1/abis/adjudication.
#[derive(Debug, Clone, serde::Deserialize, ToSchema)]
pub struct ListAdjudicationQuery {
	#[serde(default)]
	pub status: Option<String>,
	#[serde(default)]
	pub min_score: Option<f32>,
	#[serde(default)]
	pub page: Option<u32>,
	#[serde(default)]
	pub page_size: Option<u32>,
}
