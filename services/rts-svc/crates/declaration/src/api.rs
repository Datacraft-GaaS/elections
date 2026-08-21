//! Axum sub-router for the declaration crate (WP-3.3, M51).
//! Merged into rts-api's main Router at boot via Router::merge().
//!
//! Path prefix: /api/v1/declarations

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

// ---- request/response models ------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct InitiateDeclarationRequest {
    pub contest_id:        String,
    pub election_id:       String,
    pub tally_snapshot_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeclarationCreated {
    pub declaration_id: String,
    pub status:         String,
}

#[derive(Debug, Deserialize)]
pub struct DeclareRequest {
    pub declared_by:  String,
    pub ceremony_id:  String,
    pub hsm_signature: String,
}

#[derive(Debug, Deserialize)]
pub struct AmendRequest {
    pub reason:                 String,
    pub amended_by:             String,
    pub approved_by:            String,
    pub previous_snapshot_hash: String,
    pub new_snapshot_hash:      String,
    pub court_order_ref:        Option<String>,
    pub new_tally_event_id:     Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JudiciaryAckRequest {
    pub registrar_id:    String,
    pub acknowledged_at: String,
}

#[derive(Debug, Serialize)]
pub struct DeclarationState {
    pub declaration_id: String,
    pub status:         String,
    pub winner_id:      Option<String>,
    pub declared_at:    Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PetitionWindowState {
    pub declaration_id:      String,
    pub status:              String,
    pub window_open_at:      Option<String>,
    pub window_close_at:     Option<String>,
    pub petition_court_case: Option<String>,
}

// ---- handlers ---------------------------------------------------------------

async fn initiate_declaration(
    Json(req): Json<InitiateDeclarationRequest>,
) -> impl IntoResponse {
    let id = Uuid::now_v7();
    info!(
        contest_id = %req.contest_id,
        declaration_id = %id,
        "declaration: initiate"
    );
    (
        StatusCode::CREATED,
        Json(DeclarationCreated {
            declaration_id: id.to_string(),
            status: "pending".to_string(),
        }),
    )
}

async fn declare_result(
    Path(id): Path<String>,
    Json(req): Json<DeclareRequest>,
) -> impl IntoResponse {
    info!(
        declaration_id = %id,
        declared_by = %req.declared_by,
        ceremony_id = %req.ceremony_id,
        "declaration: declare"
    );
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "declaration_id": id,
            "status": "declared",
            "ceremony_id": req.ceremony_id,
        })),
    )
}

async fn set_runoff(Path(id): Path<String>) -> impl IntoResponse {
    info!(declaration_id = %id, "declaration: runoff");
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "declaration_id": id, "status": "runoff_required" })),
    )
}

async fn set_tie(Path(id): Path<String>) -> impl IntoResponse {
    info!(declaration_id = %id, "declaration: tie_pending");
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "declaration_id": id, "status": "tie_pending" })),
    )
}

async fn amend_declaration(
    Path(id): Path<String>,
    Json(req): Json<AmendRequest>,
) -> impl IntoResponse {
    info!(
        declaration_id = %id,
        amended_by = %req.amended_by,
        "declaration: amend"
    );
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "declaration_id": id,
            "amendment_id": Uuid::now_v7().to_string(),
            "status": "amended",
        })),
    )
}

async fn get_declaration(Path(id): Path<String>) -> impl IntoResponse {
    Json(DeclarationState {
        declaration_id: id,
        status: "pending".to_string(),
        winner_id: None,
        declared_at: None,
    })
}

async fn get_certificate(Path(id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({
        "declaration_id": id,
        "status": "not_issued",
    }))
}

async fn get_petition_window(Path(id): Path<String>) -> impl IntoResponse {
    Json(PetitionWindowState {
        declaration_id: id,
        status: "unopened".to_string(),
        window_open_at: None,
        window_close_at: None,
        petition_court_case: None,
    })
}

async fn judiciary_ack(
    Path(id): Path<String>,
    Json(req): Json<JudiciaryAckRequest>,
) -> impl IntoResponse {
    info!(
        declaration_id = %id,
        registrar_id = %req.registrar_id,
        "declaration: judiciary_ack"
    );
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "declaration_id": id,
            "acknowledged_at": req.acknowledged_at,
        })),
    )
}

// ---- router -----------------------------------------------------------------

/// Build the sub-router.  Call `Router::merge(declaration::api::router())`
/// from rts-api's build_router() to mount these paths.
pub fn router() -> Router {
    Router::new()
        .route("/api/v1/declarations", post(initiate_declaration))
        .route("/api/v1/declarations/:id", get(get_declaration))
        .route("/api/v1/declarations/:id/declare", put(declare_result))
        .route("/api/v1/declarations/:id/runoff", put(set_runoff))
        .route("/api/v1/declarations/:id/tie", put(set_tie))
        .route("/api/v1/declarations/:id/amend", post(amend_declaration))
        .route("/api/v1/declarations/:id/certificate", get(get_certificate))
        .route(
            "/api/v1/declarations/:id/petition-window",
            get(get_petition_window),
        )
        .route("/api/v1/declarations/:id/judiciary-ack", put(judiciary_ack))
}
