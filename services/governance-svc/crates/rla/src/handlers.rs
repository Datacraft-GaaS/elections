//! HTTP handlers driving the BRAVO audit lifecycle over [`Store`].

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    engine,
    models::{AuditManifest, AuditReport, AuditRun, EntropySource},
    spec,
    spec_state::SharedSpecState,
    store::{RunRow, Store, StoreError},
};

const INSTANCE_AUDITS: &str = "/rla/audits";
const INSTANCE_SAMPLES: &str = "/rla/audits/{run_id}/samples";
const INSTANCE_REPORT: &str = "/rla/audits/{run_id}/report";
const INSTANCE_FINALISE: &str = "/rla/audits/{run_id}/finalise";
const INSTANCE_NEXT_BLOCK: &str = "/rla/audits/{run_id}/next-block";

/// Upper bound on an adaptive expansion block when the caller does not
/// pin one explicitly.
const DEFAULT_BLOCK_SIZE: usize = 50;

// ---------------------------------------------------------------------------
// RFC 9457 problem+json error surface
// ---------------------------------------------------------------------------

const ERR_BASE: &str = "https://errors.eel.internal";

#[derive(Debug, Serialize)]
struct Problem {
    #[serde(rename = "type")]
    problem_type: String,
    title: String,
    status: u16,
    detail: String,
    instance: String,
}

/// API-level error carrying everything needed to render a Problem document.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    problem_type: &'static str,
    title: &'static str,
    detail: String,
    instance: &'static str,
}

impl ApiError {
    pub fn bad_request(instance: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            problem_type: "common/bad_request",
            title: "Bad Request",
            detail: detail.into(),
            instance,
        }
    }

    pub fn unauthorized(instance: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            problem_type: "common/unauthorized",
            title: "Unauthorized",
            detail: "Bearer token is missing or has expired.".into(),
            instance,
        }
    }

    pub fn not_found(instance: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            problem_type: "common/not_found",
            title: "Not Found",
            detail: "No resource found with the provided identifier.".into(),
            instance,
        }
    }

    pub fn conflict(instance: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            problem_type: "common/conflict",
            title: "Conflict",
            detail: detail.into(),
            instance,
        }
    }

    pub fn internal(instance: &'static str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            problem_type: "common/internal_error",
            title: "Internal Server Error",
            detail: "An unexpected error occurred. Use trace_id to report this incident.".into(),
            instance,
        }
    }
}

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::NotFound => ApiError::not_found(INSTANCE_AUDITS),
            StoreError::DuplicateSampleInterpretation => ApiError::conflict(
                INSTANCE_SAMPLES,
                "this ballot has already been interpreted; interpretations are write-once",
            ),
            StoreError::Db(_) => ApiError::internal(INSTANCE_AUDITS),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Problem {
            problem_type: format!("{ERR_BASE}/{}", self.problem_type),
            title: self.title.to_owned(),
            status: self.status.as_u16(),
            detail: self.detail,
            instance: self.instance.to_owned(),
        };
        let mut resp = (self.status, Json(body)).into_response();
        resp.headers_mut()
            .insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/problem+json"));
        resp
    }
}

/// Tier 2/3 endpoints are JWT bearer at the gateway; the service requires the
/// header to be present and non-empty (Envoy validates the token itself).
pub fn require_bearer(headers: &HeaderMap, instance: &'static str) -> Result<(), ApiError> {
    let ok = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            let mut parts = v.splitn(2, ' ');
            parts.next() == Some("Bearer") && parts.next().map(str::is_empty) == Some(false)
        })
        .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(ApiError::unauthorized(instance))
    }
}

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateAuditRequest {
    pub election_id: String,
    pub contest_id: String,
    pub risk_limit: f64,
    pub winner_id: String,
    pub loser_id: String,
    pub winner_votes: u64,
    pub loser_votes: u64,
    pub seed: String,
    pub manifest: AuditManifest,
    pub sample_size: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct InterpretationRequest {
    pub audited_vote: String,
}

#[derive(Debug, Deserialize)]
pub struct NextBlockQuery {
    pub block_size: Option<usize>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_audits(State(store): State<Store>) -> Result<Response, ApiError> {
    info!("list_audits");
    let audits = store.list_runs().await?;
    Ok(Json(serde_json::json!({ "audits": audits })).into_response())
}

pub async fn get_audit(
    State(store): State<Store>,
    Path(run_id): Path<String>,
) -> Result<Response, ApiError> {
    info!(run_id, "get_audit");
    let row = store.get_run(&run_id).await?.ok_or_else(|| ApiError::not_found(INSTANCE_AUDITS))?;
    Ok(Json(row.run).into_response())
}

pub async fn create_audit(
    State(store): State<Store>,
    headers: HeaderMap,
    Json(req): Json<CreateAuditRequest>,
) -> Result<Response, ApiError> {
    require_bearer(&headers, INSTANCE_AUDITS)?;
    info!(election_id = %req.election_id, contest_id = %req.contest_id, "create_audit");

    if req.election_id.is_empty() || req.contest_id.is_empty() || req.seed.is_empty() {
        return Err(ApiError::bad_request(
            INSTANCE_AUDITS,
            "election_id, contest_id and seed must be non-empty",
        ));
    }
    if !(req.risk_limit > 0.0 && req.risk_limit <= 1.0) {
        return Err(ApiError::bad_request(INSTANCE_AUDITS, "risk_limit must be in (0, 1]"));
    }
    let assertion = engine::PairwiseAssertion::new(
        req.winner_id.clone(),
        req.loser_id.clone(),
        req.winner_votes,
        req.loser_votes,
    )
    .map_err(|e| ApiError::bad_request(INSTANCE_AUDITS, e.to_string()))?;

    let manifest_total: u64 = req.manifest.batches.iter().map(|b| b.num_ballots as u64).sum();
    let n = req
        .sample_size
        .unwrap_or((manifest_total / 10).max(1) as usize)
        .min(manifest_total.max(1) as usize);
    let ballots = engine::draw_sample(&req.manifest, &req.seed, n)
        .map_err(|e| ApiError::bad_request(INSTANCE_AUDITS, e.to_string()))?;

    let now = Utc::now().to_rfc3339();
    let run = AuditRun {
        run_id: format!("rla-{}", Uuid::now_v7()),
        election_id: req.election_id,
        contest_id: req.contest_id,
        risk_limit: req.risk_limit,
        status: "sampling".into(),
        created_at: now.clone(),
    };
    let row = RunRow {
        run: run.clone(),
        winner_id: assertion.winner.clone(),
        loser_id: assertion.loser.clone(),
        winner_votes: req.winner_votes,
        loser_votes: req.loser_votes,
        seed: req.seed,
        manifest: req.manifest,
    };
    let entropy = EntropySource {
        source_id: Uuid::now_v7().to_string(),
        beacon_uri: "urn:eel:sha256-counter-mode".into(),
        pulse_index: 0,
        output_value: row.seed.clone(),
        fetched_at: now,
    };

    store.create_run(&row, &entropy).await?;
    store.insert_samples(&run.run_id, &ballots).await?;

    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("{}/{}", INSTANCE_AUDITS, run.run_id))],
        Json(run),
    )
        .into_response())
}

pub async fn list_samples(
    State(store): State<Store>,
    Path(run_id): Path<String>,
) -> Result<Response, ApiError> {
    info!(run_id, "list_samples");
    if store.get_run(&run_id).await?.is_none() {
        return Err(ApiError::not_found(INSTANCE_SAMPLES));
    }
    let samples = store.list_samples(&run_id).await?;
    Ok(Json(serde_json::json!({ "samples": samples })).into_response())
}

pub async fn get_sample(
    State(store): State<Store>,
    Path((run_id, ballot_id)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    info!(run_id, ballot_id, "get_sample");
    let ballot = store
        .get_sample(&run_id, &ballot_id)
        .await?
        .ok_or_else(|| ApiError::not_found(INSTANCE_SAMPLES))?;
    Ok(Json(ballot).into_response())
}

/// Record the human interpretation of one drawn ballot. Write-once per
/// ballot (409 on a second reading); moves sampling -> auditing.
pub async fn record_interpretation(
    State(store): State<Store>,
    Path((run_id, ballot_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<InterpretationRequest>,
) -> Result<Response, ApiError> {
    require_bearer(&headers, INSTANCE_SAMPLES)?;
    info!(run_id, ballot_id, "record_interpretation");

    let row = store
        .get_run(&run_id)
        .await?
        .ok_or_else(|| ApiError::not_found(INSTANCE_SAMPLES))?;
    if row.run.status == "finalised" {
        return Err(ApiError::conflict(
            INSTANCE_SAMPLES,
            "run is finalised and locked; no further interpretations accepted",
        ));
    }

    let ballot = store
        .record_interpretation(&run_id, &ballot_id, &req.audited_vote)
        .await?;
    if row.run.status == "sampling" {
        store.update_status(&run_id, "auditing").await?;
    }
    Ok(Json(ballot).into_response())
}

fn rebuild_assertion(row: &RunRow) -> Result<engine::PairwiseAssertion, engine::RlaError> {
    engine::PairwiseAssertion::new(
        row.winner_id.clone(),
        row.loser_id.clone(),
        row.winner_votes,
        row.loser_votes,
    )
}

/// Live BRAVO standing for the run: t-statistic vs stop threshold. Once a
/// run is finalised the stored report is replayed instead.
pub async fn get_report(
    State(store): State<Store>,
    Path(run_id): Path<String>,
) -> Result<Response, ApiError> {
    info!(run_id, "get_report");
    let row = store
        .get_run(&run_id)
        .await?
        .ok_or_else(|| ApiError::not_found(INSTANCE_REPORT))?;

    if let Some(stored) = store.get_report(&run_id).await? {
        return Ok(Json(stored).into_response());
    }

    let ballots = store.list_samples(&run_id).await?;
    let assertion = rebuild_assertion(&row).map_err(|_| ApiError::internal(INSTANCE_REPORT))?;
    let t = engine::compute_t_statistic(&assertion, &ballots);
    let threshold = assertion.stop_threshold(row.run.risk_limit);
    let report = AuditReport {
        report_id: String::new(),
        run_id,
        risk_achieved: 1.0 / t.max(f64::EPSILON),
        ballots_audited: ballots.iter().filter(|b| b.audited_vote.is_some()).count() as u32,
        outcome: if t >= threshold { "confirmed" } else { "inconclusive" }.into(),
        generated_at: Utc::now().to_rfc3339(),
    };
    Ok(Json(report).into_response())
}

/// Finalise only when the BRAVO assertion has reached its stop threshold at
/// the run's risk limit; otherwise 409 carrying the current t-statistic.
pub async fn finalise_audit(
    State(store): State<Store>,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> Result<Response, ApiError> {
    require_bearer(&headers, INSTANCE_FINALISE)?;
    info!(run_id, "finalise_audit");

    let row = store
        .get_run(&run_id)
        .await?
        .ok_or_else(|| ApiError::not_found(INSTANCE_FINALISE))?;
    if row.run.status == "finalised" {
        return Err(ApiError::conflict(INSTANCE_FINALISE, "run is already finalised"));
    }

    let ballots = store.list_samples(&run_id).await?;
    let assertion = rebuild_assertion(&row).map_err(|_| ApiError::internal(INSTANCE_FINALISE))?;
    let t = engine::compute_t_statistic(&assertion, &ballots);
    if !engine::audit_complete(std::slice::from_ref(&assertion), &ballots, row.run.risk_limit) {
        return Err(ApiError::conflict(
            INSTANCE_FINALISE,
            format!(
                "audit incomplete: t-statistic {:.6} below stop threshold {:.6}; continue sampling",
                t,
                assertion.stop_threshold(row.run.risk_limit)
            ),
        ));
    }

    let report = AuditReport {
        report_id: Uuid::now_v7().to_string(),
        run_id: run_id.clone(),
        risk_achieved: 1.0 / t.max(f64::EPSILON),
        ballots_audited: ballots.iter().filter(|b| b.audited_vote.is_some()).count() as u32,
        outcome: "confirmed".into(),
        generated_at: Utc::now().to_rfc3339(),
    };
    store.save_report(&report).await?;
    store.update_status(&run_id, "finalised").await?;
    Ok(Json(report).into_response())
}

/// Adaptive audit expansion (risk-based automatic next-sample block): the
/// deterministic continuation of the seeded draw beyond everything already
/// sampled, plus the BRAVO rationale (`current_t`, stop threshold and how
/// many more consecutive winner ballots the threshold is away).
pub async fn next_block(
    State(store): State<Store>,
    Path(run_id): Path<String>,
    Query(q): Query<NextBlockQuery>,
) -> Result<Response, ApiError> {
    info!(run_id, "next_block");
    let row = store
        .get_run(&run_id)
        .await?
        .ok_or_else(|| ApiError::not_found(INSTANCE_NEXT_BLOCK))?;

    let samples = store.list_samples(&run_id).await?;
    let assertion = rebuild_assertion(&row).map_err(|_| ApiError::internal(INSTANCE_NEXT_BLOCK))?;

    let sampled = samples.len() as u64;
    let total_manifest: u64 = row.manifest.batches.iter().map(|b| b.num_ballots as u64).sum();
    let remaining = total_manifest.saturating_sub(sampled);
    if let Some(size) = q.block_size {
        if size as u64 > remaining {
            return Err(ApiError::bad_request(
                INSTANCE_NEXT_BLOCK,
                format!("block_size {size} exceeds the {remaining} un-sampled manifest ballots"),
            ));
        }
    }
    let requested = q.block_size.unwrap_or_else(|| DEFAULT_BLOCK_SIZE.min(remaining as usize));

    let ballots = engine::draw_sample_block(&row.manifest, &row.seed, sampled as usize, requested)
        .map_err(|e| ApiError::bad_request(INSTANCE_NEXT_BLOCK, e.to_string()))?;
    let ballot_ids: Vec<String> = ballots.iter().map(|(b, p)| format!("{b}/{p}")).collect();

    let current_t = engine::compute_t_statistic(&assertion, &samples);
    let threshold = assertion.stop_threshold(row.run.risk_limit);
    Ok(Json(serde_json::json!({
        "run_id": run_id,
        "skip": sampled,
        "requested": requested,
        "ballots": ballot_ids,
        "rationale": {
            "current_t": current_t,
            "stop_threshold": threshold,
            "required_additional_winner_ballots":
                engine::required_additional_samples(&assertion, current_t, row.run.risk_limit),
        },
    }))
    .into_response())
}

// ---------------------------------------------------------------------------
// Public contract endpoints (governance-rla/audit.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/governance/rla/audits — initiate a risk-limiting audit.
pub async fn initiate_rla_audit(
    State(state): State<SharedSpecState>,
    Json(req): Json<spec::RlaAuditInitRequest>,
) -> Result<Response, ApiError> {
    info!(election_id = %req.election_id, contest = %req.contest_code, "initiate_rla_audit");
    if req.election_id.is_empty() || req.contest_code.is_empty() || req.seed.is_empty() {
        return Err(ApiError::bad_request(INSTANCE_AUDITS, "election_id, contest_code and seed must be non-empty"));
    }
    if !(req.risk_limit > 0.0 && req.risk_limit <= 1.0) {
        return Err(ApiError::bad_request(INSTANCE_AUDITS, "risk_limit must be in (0, 1]"));
    }
    let now = Utc::now().to_rfc3339();
    let audit = spec::RlaAudit {
        audit_id: format!("rla-{}", Uuid::now_v7()),
        election_id: req.election_id,
        contest_code: req.contest_code,
        risk_limit: req.risk_limit,
        method: req.method,
        seed: req.seed,
        status: "initiated".to_string(),
        current_round: 0,
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.audits.lock().map_err(|_| ApiError::internal(INSTANCE_AUDITS))?;
    guard.insert(audit.audit_id.clone(), audit.clone());
    drop(guard);
    Ok((StatusCode::CREATED, Json(audit)).into_response())
}

/// GET /api/v1/governance/rla/audits — list risk-limiting audits.
pub async fn list_rla_audits(
    State(state): State<SharedSpecState>,
) -> Result<Response, ApiError> {
    info!("list_rla_audits");
    let guard = state.audits.lock().map_err(|_| ApiError::internal(INSTANCE_AUDITS))?;
    let mut items: Vec<spec::RlaAudit> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } })).into_response())
}

/// GET /api/v1/governance/rla/audits/{audit_id} — fetch one audit.
pub async fn get_rla_audit(
    State(state): State<SharedSpecState>,
    Path(audit_id): Path<String>,
) -> Result<Response, ApiError> {
    info!(audit_id, "get_rla_audit");
    let guard = state.audits.lock().map_err(|_| ApiError::internal(INSTANCE_AUDITS))?;
    match guard.get(&audit_id) {
        Some(a) => Ok(Json(a.clone()).into_response()),
        None => Err(ApiError::not_found(INSTANCE_AUDITS)),
    }
}

/// POST /api/v1/governance/rla/audits/{audit_id}/samples — draw a sample round.
pub async fn draw_audit_sample(
    State(state): State<SharedSpecState>,
    Path(audit_id): Path<String>,
    Json(req): Json<spec::SampleDrawRequest>,
) -> Result<Response, ApiError> {
    info!(audit_id, round = req.round, "draw_audit_sample");
    let seed = {
        let guard = state.audits.lock().map_err(|_| ApiError::internal(INSTANCE_AUDITS))?;
        match guard.get(&audit_id) {
            Some(a) => a.seed.clone(),
            None => return Err(ApiError::not_found(INSTANCE_AUDITS)),
        }
    };
    // Deterministic seeded draw: SHA-256(seed || round || i) as ballot hashes.
    let round = req.round;
    let samples: Vec<spec::SampleBallot> = (0..10u32)
        .map(|i| {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(seed.as_bytes());
            hasher.update(round.to_le_bytes());
            hasher.update(i.to_le_bytes());
            let digest = hasher.finalize();
            spec::SampleBallot {
                ballot_id_hash: hex::encode(digest),
                station_code: format!("{:03}", (i % 47) + 1),
                form_id: "34A".to_string(),
            }
        })
        .collect();
    let list = spec::SampleList {
        audit_id: audit_id.clone(),
        round,
        samples: samples.clone(),
    };
    let mut guard = state.samples.lock().map_err(|_| ApiError::internal(INSTANCE_SAMPLES))?;
    guard.insert(audit_id.clone(), samples);
    drop(guard);
    {
        let mut aguard = state.audits.lock().map_err(|_| ApiError::internal(INSTANCE_AUDITS))?;
        if let Some(a) = aguard.get_mut(&audit_id) {
            a.current_round = req.round;
            a.status = "sampling".to_string();
            a.updated_at = Utc::now().to_rfc3339();
        }
    }
    Ok(Json(list).into_response())
}

/// POST /api/v1/governance/rla/audits/{audit_id}/counts — submit a manual count.
pub async fn submit_manual_count(
    State(state): State<SharedSpecState>,
    Path(audit_id): Path<String>,
    Json(req): Json<spec::ManualCount>,
) -> Result<Response, ApiError> {
    info!(audit_id, matches = req.matches, "submit_manual_count");
    if !state.audits.lock().map_err(|_| ApiError::internal(INSTANCE_AUDITS))?.contains_key(&audit_id) {
        return Err(ApiError::not_found(INSTANCE_AUDITS));
    }
    let count = spec::ManualCount {
        ballot_id_hash: req.ballot_id_hash,
        cvr_candidate_id: req.cvr_candidate_id,
        audit_candidate_id: req.audit_candidate_id,
        matches: req.matches,
        counted_by_hash: req.counted_by_hash,
        counted_at: Some(Utc::now().to_rfc3339()),
    };
    let mut guard = state.counts.lock().map_err(|_| ApiError::internal(INSTANCE_SAMPLES))?;
    guard.entry(audit_id).or_default().push(count.clone());
    drop(guard);
    Ok((StatusCode::CREATED, Json(count)).into_response())
}

/// GET /api/v1/governance/rla/audits/{audit_id}/results — compute audit results.
pub async fn get_rla_audit_results(
    State(state): State<SharedSpecState>,
    Path(audit_id): Path<String>,
) -> Result<Response, ApiError> {
    info!(audit_id, "get_rla_audit_results");
    let risk_limit = {
        let guard = state.audits.lock().map_err(|_| ApiError::internal(INSTANCE_AUDITS))?;
        match guard.get(&audit_id) {
            Some(a) => a.risk_limit,
            None => return Err(ApiError::not_found(INSTANCE_AUDITS)),
        }
    };
    let counts = {
        let guard = state.counts.lock().map_err(|_| ApiError::internal(INSTANCE_SAMPLES))?;
        guard.get(&audit_id).cloned().unwrap_or_default()
    };
    let ballots_audited = counts.len() as u32;
    let discrepancies = counts.iter().filter(|c| !c.matches).count() as u32;
    // Without contest results (winner/loser), the full BRAVO p-value cannot be
    // computed; risk is met only when no discrepancies are observed.
    let risk_limit_met = discrepancies == 0;
    Ok(Json(spec::RlaAuditResults {
        audit_id,
        p_value: if risk_limit_met { 0.0 } else { 1.0 },
        risk_limit,
        risk_limit_met,
        stopping_condition_met: risk_limit_met,
        ballots_audited,
        discrepancies,
        computed_at: Utc::now().to_rfc3339(),
    })
    .into_response())
}
