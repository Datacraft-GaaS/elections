use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{
        AutoReferralPacket, CorrectionClaim, CorrectionStatus, CorrectionType, Determination,
        Objection, ObjectionStatus, RollSnapshot,
    },
    events,
};

#[derive(Clone, Default)]
pub struct AppState {
    pub snapshots: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::domain::RollSnapshot>>>,
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
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::debug!(%id, "get roll snapshot");
    StatusCode::NOT_FOUND
}

async fn roll_lookup(
    State(_state): State<AppState>,
    Json(req): Json<RollLookupReq>,
) -> impl IntoResponse {
    // Constant-time shape: response always has same JSON structure.
    // Real impl delegates to search::constant_time_lookup.
    let _ = req.step_up_token; // validated in middleware
    tracing::debug!("roll lookup (constant-time stub)");
    (
        StatusCode::OK,
        Json(RollLookupResp {
            found: false,
            station_id: None,
        }),
    )
}

async fn file_objection(
    State(_state): State<AppState>,
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
    (StatusCode::CREATED, Json(obj))
}

async fn get_objection(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::debug!(%id, "get objection");
    StatusCode::NOT_FOUND
}

async fn update_objection_status(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_req): Json<UpdateObjectionStatusReq>,
) -> impl IntoResponse {
    tracing::info!(%id, "objection status updated");
    StatusCode::NO_CONTENT
}

async fn file_correction(
    State(_state): State<AppState>,
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
    (StatusCode::CREATED, Json(claim))
}

async fn get_correction(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::debug!(%id, "get correction");
    StatusCode::NOT_FOUND
}

async fn decide_correction(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<DecideCorrectionReq>,
) -> impl IntoResponse {
    let det = Determination {
        id: Uuid::now_v7(),
        claim_id: id,
        decision: req.decision,
        decided_by: req.decided_by,
        decided_at: Utc::now(),
    };
    tracing::info!(claim_id = %id, "correction determination recorded");
    (StatusCode::OK, Json(det))
}

async fn run_auto_referral(State(_state): State<AppState>) -> impl IntoResponse {
    // Idempotent: real impl queries DB for pending sustained corrections
    // and delegates to auto_referral::AutoReferralScheduler.
    tracing::info!("auto-referral run triggered (stub)");
    StatusCode::NO_CONTENT
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

async fn search_voter_list(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({ "found": false, "polling_station_name": serde_json::Value::Null }))
}
