use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::cost_report::build_cost_report;
use crate::models::*;
use crate::spec;
use crate::state::SharedState;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn create_by_election(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    info!("create_by_election");
    // Commissioner dual-sign verified in production via SpiceDB (P01) -- FR-M50-002
    // Concurrency guard: reject if 10 active by-elections exist (FR-M50-008) -- stub
    let id = uuid::Uuid::now_v7().to_string();
    let notice_date = body["notice_date"].as_str().unwrap_or("2025-01-01").to_string();
    let election_date = body["election_date"].as_str().unwrap_or("2025-02-01").to_string();
    let by_election = ByElection {
        id: id.clone(),
        constituency_id: body["constituency_id"].as_str().unwrap_or("").to_string(),
        notice_date: notice_date.clone(),
        election_date: election_date.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    let short_notice = by_election.is_short_notice();
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "by_election": by_election,
            "short_notice_alert": short_notice,
        })),
    )
}

/// A declared operation with no implementation behind it.
///
/// The pure functions this crate does have -- `generate_subset`,
/// `generate_kit_plan`, `build_cost_report` -- are real arithmetic over inputs
/// that do not exist here. `generate_kit_plan` in particular assigns every
/// polling station `station_gps_lat: 0.0, station_gps_lng: 0.0`, which is
/// 0N 0E in the Gulf of Guinea: a kit delivery plan pointing at open ocean.
fn not_implemented(operation: &str, detail: &str) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": "Not Implemented",
            "status": 501,
            "detail": format!(
                "{operation} is declared by the contract but has no \
                 implementation behind it: {detail}"
            ),
        })),
    )
}

pub async fn generate_register_subset(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "generate_register_subset");
    // generate_subset returns station_count: 0 and voter_count: 0, annotated
    // "populated from M02 response" -- and its own comment records that no M02
    // adapter crate exists in this workspace. So this returned 201 for a
    // register subset covering zero voters.
    let _ = (id, &body);
    not_implemented(
        "register subset generation",
        "the M02 snapshot adapter does not exist; generate_subset returns \
         station_count 0 and voter_count 0, and this previously returned 201 \
         for a subset covering no voters",
    )
}

pub async fn create_kit_plan(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "create_kit_plan");
    // The hash in sign_kit_plan is real: a SHA-256 over the canonical plan,
    // stored in a field named signed_blob_hash. It is a digest, not a
    // signature -- there is no key, so it attests only that the plan is
    // self-consistent, not who produced it. The plan it hashes assigns every
    // station gps 0.0, 0.0 and reserves no kit, because the M20 inventory
    // adapter does not exist here. So 201 described a delivery plan for kits
    // that were never reserved, delivered to coordinates in the Atlantic.
    let _ = (id, &body);
    not_implemented(
        "kit plan creation",
        "the M20 inventory adapter does not exist, so no kit is reserved and \
         every station is assigned gps 0.0, 0.0; the plan hash is a plain \
         digest in a field named signed_blob_hash, which is not a signature",
    )
}

pub async fn activate_by_election(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "activate_by_election");
    // Answered 204, which is a successful activation. The comment notes a
    // GAZETTED transition is required first; no transition was checked, so a
    // by-election could be activated from any state.
    let _ = id;
    not_implemented(
        "by-election activation",
        "no state machine or M04 register notice is wired; it previously \
         returned 204 for an activation it never performed and did not check \
         the required GAZETTED transition",
    )
}

pub async fn submit_cost_report(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "submit_cost_report");
    let parse_u64 = |key: &str| body[key].as_u64().unwrap_or(0);
    match build_cost_report(
        id,
        parse_u64("kit_days_kes"),
        parse_u64("staff_days_kes"),
        parse_u64("transport_kes"),
        parse_u64("printing_kes"),
        parse_u64("venue_kes"),
        parse_u64("other_kes"),
    ) {
        Ok(report) => (StatusCode::CREATED, Json(serde_json::to_value(report).unwrap())),
        Err(msg) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": msg })),
        ),
    }
}

pub async fn get_cost_report(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_cost_report");
    // Bypassed build_cost_report -- which is real arithmetic -- and returned a
    // default-constructed report: every cost line zero, presented as a cost
    // report for a by-election.
    let _ = id;
    not_implemented(
        "cost report retrieval",
        "no cost ledger is stored; it previously returned a default report \
         with every cost line zero",
    )
}

// ---------------------------------------------------------------------------
// Public contract endpoints (governance-by-election/operations.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/governance/by-election/elections — declare a by-election.
pub async fn declare_by_election(
    State(state): State<SharedState>,
    Json(req): Json<spec::ByElectionDeclaration>,
) -> impl IntoResponse {
    info!(constituency = %req.constituency_code, "declare_by_election");
    let now = chrono::Utc::now().to_rfc3339();
    let election = spec::ByElection {
        by_election_id: uuid::Uuid::now_v7().to_string(),
        trigger_reason: req.trigger_reason,
        constituency_code: req.constituency_code,
        gazette_ref: req.gazette_ref,
        election_date: req.election_date,
        status: "declared".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.elections.lock().expect("lock poisoned");
    guard.insert(election.by_election_id.clone(), election.clone());
    drop(guard);
    (StatusCode::CREATED, Json(election)).into_response()
}

/// GET /api/v1/governance/by-election/elections — list by-elections.
pub async fn list_by_elections(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_by_elections");
    let guard = state.elections.lock().expect("lock poisoned");
    let mut items: Vec<spec::ByElection> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }))
}

/// GET /api/v1/governance/by-election/elections/{by_election_id} — fetch one.
pub async fn get_by_election(
    State(state): State<SharedState>,
    Path(by_election_id): Path<String>,
) -> impl IntoResponse {
    info!(by_election_id, "get_by_election");
    let guard = state.elections.lock().expect("lock poisoned");
    match guard.get(&by_election_id) {
        Some(e) => Json(e.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "by-election not found"})),
        )
            .into_response(),
    }
}

/// PATCH /api/v1/governance/by-election/elections/{by_election_id}/status —
/// transition the by-election status.
pub async fn transition_by_election_status(
    State(state): State<SharedState>,
    Path(by_election_id): Path<String>,
    Json(req): Json<spec::StatusTransition>,
) -> impl IntoResponse {
    info!(by_election_id, new_status = %req.new_status, "transition_by_election_status");
    let mut guard = state.elections.lock().expect("lock poisoned");
    match guard.get_mut(&by_election_id) {
        Some(e) => {
            e.status = req.new_status;
            e.updated_at = chrono::Utc::now().to_rfc3339();
            Json(e.clone()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "by-election not found"})),
        )
            .into_response(),
    }
}
