//! eel portal-api: public results, Merkle proofs, bulk downloads (M11, WP-3.9).
//!
//! Endpoints (all under /api/v1/portal):
//!   GET  /snapshots/latest          - latest signed snapshot
//!   GET  /snapshots/{id}            - snapshot by ID
//!   GET  /snapshots/bulk.tar.gz     - redirect to MinIO signed URL
//!   GET  /snapshots/{id}/verify     - Merkle proof + STH for a snapshot
//!   GET  /health                    - liveness probe
//!
//! Rate limiting, WAF, and HSTS are enforced at the edge (ADR-0010).
//! No PII is returned from any endpoint.
#![warn(missing_docs)]

use axum::{
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Response types (data class C/D, no PII per FR-M11-026)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SnapshotSummary {
    pub snapshot_id: Uuid,
    pub merkle_root: String,
    pub sig: String,
    pub published_at: DateTime<Utc>,
    pub scope: ResultScope,
    pub scope_code: Option<String>,
    pub position_code: String,
    pub stale: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultScope {
    National,
    Presidential,
    County,
    Constituency,
    Ward,
    CountyAssembly,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MerkleProofResponse {
    pub snapshot_id: Uuid,
    pub leaf_index: u64,
    pub leaf_hash: String,
    pub siblings: Vec<String>,
    pub root: String,
    pub sth_sig: String,
    pub tree_size: u64,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HealthResponse {
    pub ok: bool,
    pub service: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProblemDetail {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
}

impl ProblemDetail {
    pub fn not_found(detail: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::NOT_FOUND,
            Json(Self {
                problem_type: "https://eel.local/problems/not-found".into(),
                title: "Not Found".into(),
                status: 404,
                detail: detail.into(),
            }),
        )
    }

    pub fn bad_request(detail: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::BAD_REQUEST,
            Json(Self {
                problem_type: "https://eel.local/problems/bad-request".into(),
                title: "Bad Request".into(),
                status: 400,
                detail: detail.into(),
            }),
        )
    }
}

// ---------------------------------------------------------------------------
// Signing response headers (FR-M11-025)
// ---------------------------------------------------------------------------

fn signing_headers(merkle_root: &str, snapshot_id: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    // These values are populated by the actual signing layer;
    // placeholders here for the routing/test layer.
    headers.insert(
        "X-Eel-Merkle-Root",
        HeaderValue::from_str(merkle_root).unwrap_or(HeaderValue::from_static("none")),
    );
    headers.insert(
        "X-Eel-Snapshot-Id",
        HeaderValue::from_str(snapshot_id).unwrap_or(HeaderValue::from_static("none")),
    );
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub mod spec;

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        service: "portal-api",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn snapshots_latest() -> impl IntoResponse {
    // In production this queries Valkey read-through -> PostgreSQL.
    // Stub returns a well-typed placeholder so cargo build --release passes.
    let snap = SnapshotSummary {
        snapshot_id: Uuid::nil(),
        merkle_root: "0000000000000000000000000000000000000000000000000000000000000000".into(),
        sig: "".into(),
        published_at: Utc::now(),
        scope: ResultScope::National,
        scope_code: None,
        position_code: "presidential".into(),
        stale: false,
    };
    let headers = signing_headers(&snap.merkle_root, &snap.snapshot_id.to_string());
    (StatusCode::OK, headers, Json(snap))
}

async fn snapshot_by_id(Path(id): Path<String>) -> impl IntoResponse {
    let Ok(uid) = id.parse::<Uuid>() else {
        return ProblemDetail::bad_request("snapshot_id must be a valid UUID").into_response();
    };
    let snap = SnapshotSummary {
        snapshot_id: uid,
        merkle_root: "0000000000000000000000000000000000000000000000000000000000000000".into(),
        sig: "".into(),
        published_at: Utc::now(),
        scope: ResultScope::National,
        scope_code: None,
        position_code: "presidential".into(),
        stale: false,
    };
    let headers = signing_headers(&snap.merkle_root, &snap.snapshot_id.to_string());
    (StatusCode::OK, headers, Json(snap)).into_response()
}

async fn snapshots_bulk() -> impl IntoResponse {
    // Redirects to a MinIO pre-signed URL. In production the URL is generated
    // per-request with a short TTL (ADR-0011 Tier 4). Stub returns 302.
    Redirect::temporary("https://minio.eel.internal/portal-snapshots/latest.tar.gz")
}

async fn snapshot_verify(Path(id): Path<String>) -> impl IntoResponse {
    let Ok(uid) = id.parse::<Uuid>() else {
        return ProblemDetail::bad_request("snapshot_id must be a valid UUID").into_response();
    };
    let proof = MerkleProofResponse {
        snapshot_id: uid,
        leaf_index: 0,
        leaf_hash: "0000000000000000000000000000000000000000000000000000000000000000".into(),
        siblings: vec![],
        root: "0000000000000000000000000000000000000000000000000000000000000000".into(),
        sth_sig: "".into(),
        tree_size: 1,
        verified: false,
    };
    (StatusCode::OK, Json(proof)).into_response()
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn app() -> Router {
    Router::new()
        .route("/api/v1/portal/health", get(health))
        .route("/api/v1/portal/snapshots/latest", get(snapshots_latest))
        .route("/api/v1/portal/snapshots/bulk.tar.gz", get(snapshots_bulk))
        .route("/api/v1/portal/snapshots/{id}", get(snapshot_by_id))
        .route("/api/v1/portal/snapshots/{id}/verify", get(snapshot_verify))
        // Public contract endpoints (portal/public.yaml)
        .route("/api/v1/portal/results", get(spec::get_national_results))
        .route("/api/v1/portal/results/{scope}/{code}", get(spec::get_scoped_results))
        .route("/api/v1/portal/forms/{form_id}/proof", get(spec::get_form_inclusion_proof))
        .route("/api/v1/portal/merkle/{root_id}", get(spec::get_merkle_root))
        .route("/api/v1/portal/turnout", get(spec::get_live_turnout))
        .route("/api/v1/portal/civic/{item_id}", get(spec::get_civic_content))
        .route("/api/v1/portal/admin/content", axum::routing::post(spec::publish_civic_content))
}
