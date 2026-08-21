//! API request/response models for the ABIS service.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Fingerprint,
    Iris,
    Face,
    Multimodal,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchOutcome {
    Match,
    NoMatch,
    Inconclusive,
    Adjudicate,
    InsufficientQuality,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Match1NRequest {
    pub probe_ref: String,
    pub modality: Modality,
    pub caller_id: String,
    pub election_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Match11Request {
    pub probe_ref: String,
    pub gallery_ref: String,
    pub modality: Modality,
    pub caller_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchAccepted {
    pub request_id: String,
    pub status: String,
    pub estimated_completion_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Match1NResult {
    pub request_id: String,
    pub status: String,
    pub candidates: Vec<MatchCandidate>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchCandidate {
    pub gallery_ref: String,
    pub score: f32,
    pub decision: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Match11Result {
    pub request_id: String,
    pub decision: String,
    pub score: f32,
    pub threshold_used: f32,
    pub processed_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjudicationStatus {
    Open,
    InReview,
    Resolved,
    Escalated,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdjudicationCase {
    pub case_id: String,
    pub status: AdjudicationStatus,
    pub match_score: f32,
    pub probe_ref: String,
    pub candidate_gallery_ref: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdjudicationCasePage {
    pub cases: Vec<AdjudicationCase>,
    pub total: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdjudicationResolutionRequest {
    pub outcome: String,
    pub adjudicator_id: String,
    pub resolution_notes: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchDedupRequest {
    pub election_id: String,
    pub mode: String,
    pub requester_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchJobAccepted {
    pub job_id: String,
    pub status: String,
    pub estimated_duration_s: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AbisReport {
    pub generated_at: String,
    pub gallery_size: u64,
    pub pending_dedup_count: u64,
    pub open_adjudication_count: u32,
    pub match_throughput_per_hour: u64,
    pub duplicate_rate_pct: f32,
    pub p99_match_ms: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthResponse {
    pub ok: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListAdjudicationQuery {
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
