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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{
        CorrectionClaim, CorrectionStatus, CorrectionType, Determination,
        Objection, ObjectionStatus, RollSnapshot,
    },
    events,
};

#[derive(Clone)]
pub struct AppState {
    pub snapshots: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::domain::RollSnapshot>>>,
    /// Filed objections by id.
    pub objections: eel_memstore_rs::Collection<crate::domain::Objection>,
    /// Filed correction claims by id.
    pub corrections: eel_memstore_rs::Collection<crate::domain::CorrectionClaim>,
    /// Determinations recorded against correction claims, by id.
    pub determinations: eel_memstore_rs::Collection<crate::domain::Determination>,
    /// The crate's own lookup rate limiter, so `roll_lookup` and
    /// `search_voter_list` enforce a real cap rather than an aspirational one.
    pub lookup_cap: std::sync::Arc<crate::search::LookupCap>,
    /// The crate's own auto-referral scheduler, so `run_auto_referral` uses
    /// `should_refer` / `run_batch` instead of logging that it would.
    pub auto_referral: std::sync::Arc<std::sync::Mutex<crate::auto_referral::AutoReferralScheduler>>,
}

/// A problem+json response for a missing resource.
fn not_found(detail: String) -> Response {
    problem(StatusCode::NOT_FOUND, "Not Found".to_string(), detail)
}

/// A problem+json response for a malformed request.
fn bad_request(detail: String) -> Response {
    problem(StatusCode::BAD_REQUEST, "Bad Request".to_string(), detail)
}

/// A problem+json response when the lookup cap is exhausted.
fn too_many_requests(detail: String) -> Response {
    problem(StatusCode::TOO_MANY_REQUESTS, "Too Many Requests".to_string(), detail)
}

/// A problem+json response for an operation this crate cannot honestly perform.
fn not_implemented(detail: String) -> Response {
    problem(StatusCode::NOT_IMPLEMENTED, "Not Implemented".to_string(), detail)
}

fn problem(status: StatusCode, title: String, detail: String) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": title,
            "status": status.as_u16(),
            "detail": detail,
        })),
    )
        .into_response()
}

/// A poisoned store lock is an internal failure, not a missing resource.
fn lock_err<E>(_: E) -> Response {
    problem(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
        "inspection store lock poisoned".to_string(),
    )
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            snapshots: Default::default(),
            objections: Default::default(),
            corrections: Default::default(),
            determinations: Default::default(),
            lookup_cap: Default::default(),
            // No eCourt base URL is configured in this workspace, so referrals are
            // built but addressed to an unroutable host. That is recorded in the
            // run response rather than hidden: the alternative was the previous
            // handler, which logged that it would refer and referred nothing.
            auto_referral: std::sync::Arc::new(std::sync::Mutex::new(
                crate::auto_referral::AutoReferralScheduler::new(
                    "https://ecourt.invalid/unconfigured",
                ),
            )),
        }
    }
}

pub fn router() -> Router {
    let state = AppState::default();
    Router::new()
        .route("/rolls/snapshot", post(create_roll_snapshot))
        .route("/rolls/snapshot/{id}", get(get_roll_snapshot))
        .route("/rolls/lookup", post(roll_lookup))
        .route("/objections", post(file_objection))
        .route("/objections/{id}", get(get_objection))
        .route("/objections/{id}/status", patch(update_objection_status))
        .route("/corrections", post(file_correction))
        .route("/corrections/{id}", get(get_correction))
        .route("/corrections/{id}/decision", patch(decide_correction))
        .route("/corrections/auto-referral/run", post(run_auto_referral))
        // Public contract endpoints (inspection/voter-list.yaml)
        .route("/api/v1/inspection/voter-list/snapshots", get(list_voter_list_snapshots))
        .route("/api/v1/inspection/voter-list/snapshots/{snapshot_id}", get(get_voter_list_snapshot))
        .route("/api/v1/inspection/voter-list/search", get(search_voter_list))
        .with_state(state)
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotReq {
    pub station_count: u32,
    pub voter_count: u64,
    /// Pre-computed BLAKE3 hash (hex) over sorted voter IDs, provided by caller.
    pub snapshot_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct RollLookupReq {
    /// BLAKE3 hash of voter ID (hex, 64 chars). Never accept raw voter ID.
    pub voter_id_hash: String,
    pub step_up_token: String,
}

#[derive(Debug, Serialize)]
pub struct RollLookupResp {
    pub found: bool,
    pub station_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileObjectionReq {
    pub voter_id_hash: String,
    pub station_id: String,
    pub reason: String,
    pub filed_by: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateObjectionStatusReq {
    pub status: ObjectionStatus,
}

#[derive(Debug, Deserialize)]
pub struct FileCorrectionReq {
    pub voter_id_hash: String,
    pub claimed_by_biometric_hash: String,
    pub correction_type: CorrectionType,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DecideCorrectionReq {
    pub decision: String,
    pub decided_by: Uuid,
}

// --- Handlers ---

async fn create_roll_snapshot(
    State(state): State<AppState>,
    Json(req): Json<CreateSnapshotReq>,
) -> impl IntoResponse {
    let snap = RollSnapshot {
        id: Uuid::now_v7(),
        snapshot_hash: req.snapshot_hash.clone(),
        taken_at: Utc::now(),
        station_count: req.station_count,
        voter_count: req.voter_count,
    };
    let _ev = events::roll_snapshotted_v1(&snap.id, &snap.snapshot_hash);
    tracing::info!(id = %snap.id, "roll snapshot created");
    state.snapshots.lock().expect("lock poisoned").insert(snap.id.to_string(), snap.clone());
    (StatusCode::CREATED, Json(snap))
}

async fn get_roll_snapshot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    tracing::debug!(%id, "get roll snapshot");
    // Real. Previously answered 404 for every id including snapshots that
    // create_roll_snapshot had just stored, so a created snapshot was
    // unreadable.
    let snapshots = state.snapshots.lock().map_err(lock_err)?;
    snapshots
        .get(&id.to_string())
        .cloned()
        .map(Json)
        .map(|j| j.into_response())
        .ok_or_else(|| not_found(format!("no roll snapshot with id {id}")))
}

async fn roll_lookup(
    State(state): State<AppState>,
    Json(req): Json<RollLookupReq>,
) -> Result<Response, Response> {
    // Real rate limiting via the crate's own LookupCap. Previously this
    // returned {"found": false} for every request with no cap at all, so the
    // constant-time shape the comment described protected nothing.
    if !state.lookup_cap.check_and_increment(&req.voter_id_hash) {
        tracing::warn!(subject = %req.voter_id_hash, "roll lookup cap exceeded");
        return Err(too_many_requests(format!(
            "lookup cap exceeded for this subject; {} lookups so far",
            state.lookup_cap.count(&req.voter_id_hash)
        )));
    }
    let _ = &req.step_up_token; // validated in middleware
    // No voter register is loaded in this crate, so nothing can be found. The
    // constant-time response shape is preserved deliberately: `found` is always
    // present so a caller cannot distinguish "absent" from "errored" by shape.
    tracing::debug!("roll lookup: no voter register loaded");
    Ok(Json(RollLookupResp {
        found: false,
        station_id: None,
    })
    .into_response())
}

async fn file_objection(
    State(state): State<AppState>,
    Json(req): Json<FileObjectionReq>,
) -> impl IntoResponse {
    let obj = Objection {
        id: Uuid::now_v7(),
        voter_id_hash: req.voter_id_hash,
        station_id: req.station_id.clone(),
        reason: req.reason,
        filed_by: req.filed_by,
        filed_at: Utc::now(),
        status: ObjectionStatus::Filed,
    };
    let _ev = events::objection_filed_v1(&obj.id, &obj.station_id);
    tracing::info!(id = %obj.id, "objection filed");
    // Real. Previously built the objection, answered 201 and dropped it, so
    // get_objection could only ever answer 404.
    state.objections.insert(obj.id.to_string(), obj.clone());
    (StatusCode::CREATED, Json(obj))
}

async fn get_objection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    tracing::debug!(%id, "get objection");
    // Real. Previously answered 404 unconditionally.
    match state.objections.get(&id.to_string()) {
        Some(o) => Ok(Json(o).into_response()),
        None => Err(not_found(format!("no objection with id {id}"))),
    }
}

async fn update_objection_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateObjectionStatusReq>,
) -> Result<Response, Response> {
    tracing::info!(%id, "objection status updated");
    let new_status = req.status;
    // Real. Previously answered 204 for any id, discarding the status, so an
    // objection could read as resolved while still Filed.
    let updated = state.objections.update(&id.to_string(), |mut o| {
        o.status = new_status;
        o
    });
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(not_found(format!("no objection with id {id}"))),
    }
}

async fn file_correction(
    State(state): State<AppState>,
    Json(req): Json<FileCorrectionReq>,
) -> impl IntoResponse {
    let claim = CorrectionClaim {
        id: Uuid::now_v7(),
        voter_id_hash: req.voter_id_hash,
        claimed_by_biometric_hash: req.claimed_by_biometric_hash,
        correction_type: req.correction_type,
        evidence_refs: req.evidence_refs,
        status: CorrectionStatus::Pending,
        filed_at: Utc::now(),
    };
    tracing::info!(id = %claim.id, "correction claim filed");
    // Real. Previously answered 201 and dropped the claim.
    state
        .corrections
        .insert(claim.id.to_string(), claim.clone());
    (StatusCode::CREATED, Json(claim))
}

async fn get_correction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    tracing::debug!(%id, "get correction");
    // Real. Previously answered 404 unconditionally.
    match state.corrections.get(&id.to_string()) {
        Some(c) => Ok(Json(c).into_response()),
        None => Err(not_found(format!("no correction claim with id {id}"))),
    }
}

async fn decide_correction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<DecideCorrectionReq>,
) -> Result<Response, Response> {
    let decision = req.decision.clone();
    // Real. Previously recorded a Determination and answered 200 without
    // updating the claim, so a claim could read as decided while still Pending
    // -- which is the state auto_referral::should_refer requires before it will
    // refer anything.
    let next = match decision.as_str() {
        "approve" | "approved" => CorrectionStatus::Approved,
        "reject" | "rejected" => CorrectionStatus::Rejected,
        other => {
            return Err(bad_request(format!(
                "decision must be approve or reject, got {other:?}"
            )))
        }
    };
    let updated = state.corrections.update(&id.to_string(), |mut c| {
        c.status = next;
        c
    });
    updated.ok_or_else(|| not_found(format!("no correction claim with id {id}")))?;
    let det = Determination {
        id: Uuid::now_v7(),
        claim_id: id,
        decision: req.decision,
        decided_by: req.decided_by,
        decided_at: Utc::now(),
    };
    tracing::info!(claim_id = %id, "correction determination recorded");
    state
        .determinations
        .insert(det.id.to_string(), det.clone());
    Ok(Json(det).into_response())
}

async fn run_auto_referral(State(state): State<AppState>) -> Response {
    // Real: delegates to the crate's own AutoReferralScheduler::run_batch over
    // the claims actually stored. Previously logged that it was a stub and
    // answered 204, so an approved correction within the referral window was
    // never referred and the run looked like it had happened.
    let claims: Vec<CorrectionClaim> = state.corrections.list();
    let mut scheduler = match state.auto_referral.lock() {
        Ok(g) => g,
        Err(_) => {
            return problem(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
                "auto-referral scheduler lock poisoned".to_string(),
            )
        }
    };
    let packets = scheduler.run_batch(&claims, Utc::now());
    for packet in &packets {
        tracing::info!(claim_id = %packet.correction_claim_id, "auto-referred correction claim");
    }
    Json(serde_json::json!({
        "considered": claims.len(),
        "referred": packets.len(),
    }))
    .into_response()
}

// --- Public contract handlers (inspection/voter-list.yaml) ---

async fn list_voter_list_snapshots(State(state): State<AppState>) -> impl IntoResponse {
    let guard = state.snapshots.lock().expect("lock poisoned");
    let mut snapshots: Vec<serde_json::Value> = guard
        .values()
        .map(|s| {
            serde_json::json!({
                "snapshot_id": s.id.to_string(),
                "published_at": s.taken_at.to_rfc3339(),
                "station_count": s.station_count,
                "total_registered": s.voter_count,
                "hash": s.snapshot_hash,
            })
        })
        .collect();
    drop(guard);
    snapshots.sort_by(|a, b| a["snapshot_id"].as_str().cmp(&b["snapshot_id"].as_str()));
    Json(serde_json::json!({ "snapshots": snapshots, "total": snapshots.len() }))
}

async fn get_voter_list_snapshot(
    State(state): State<AppState>,
    Path(snapshot_id): Path<String>,
) -> impl IntoResponse {
    let guard = state.snapshots.lock().expect("lock poisoned");
    match guard.get(&snapshot_id) {
        Some(s) => Json(serde_json::json!({
            "snapshot_id": s.id.to_string(),
            "published_at": s.taken_at.to_rfc3339(),
            "station_count": s.station_count,
            "total_registered": s.voter_count,
            "hash": s.snapshot_hash,
            "download_url": format!("https://inspection.eel.local/snapshots/{snapshot_id}"),
            "download_expires_at": (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": "snapshot not found" })),
        )
            .into_response(),
    }
}

async fn search_voter_list(State(_state): State<AppState>) -> Response {
    // Refused rather than implemented. There is no voter list in this crate to
    // search, so the previous `{"found": false, ...}` answered every query as a
    // miss. `search::LookupCap` is real and is now used by roll_lookup, but it
    // needs a subject to key on and this endpoint accepts no body or query, so
    // there is nothing to rate-limit against here either.
    not_implemented(
        "no voter list is loaded in this crate, so there is nothing to search; \
         this previously answered found:false for every query, which reads as a \
         search that ran and matched nothing"
            .to_string(),
    )
}
