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
    // X-Content-Type-Options moved out of here and onto the router as a layer: it
    // was set only on snapshot responses, which meant every other response in the
    // service -- including a JSON problem body, which a browser is most likely to
    // be pointed at -- went out without it.
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

/// The latest published result snapshot.
///
/// This returned `snapshot_id: Uuid::nil()`, `merkle_root: 64 zeros`, `sig: ""`
/// and `stale: false`, under the comment "Stub returns a well-typed placeholder so
/// cargo build --release passes." `Uuid::nil()` is the all-zero UUID and
/// `stale: false` asserts freshness — so the response said, with full confidence,
/// that the newest snapshot is the one that does not exist and is not stale.
///
/// The `X-Eel-Merkle-Root` header matters most here. The signing_headers helper
/// exists so edge caches key on the root and snapshot id (AC-M11-05); publishing a
/// zero root in that header under `stale: false` is a cache key that maps every
/// snapshot to the same value.
async fn snapshots_latest() -> axum::response::Response {
    snapshot_unavailable(
        "the latest published snapshot comes from Valkey read-through to PostgreSQL, and \
         neither is in this workspace. This handler previously returned Uuid::nil() with a \
         64-zero merkle_root, an empty signature and stale false, which asserted that the newest \
         snapshot does not exist and is not stale -- in an X-Eel-Merkle-Root header that \
         edge caches are meant to key on",
    )
}

/// A snapshot by id.
///
/// Same placeholder as [`snapshots_latest`], with the caller's own id echoed back so
/// the response looks addressed.
async fn snapshot_by_id(Path(id): Path<String>) -> axum::response::Response {
    if id.parse::<Uuid>().is_err() {
        return ProblemDetail::bad_request("snapshot_id must be a valid UUID").into_response();
    }
    snapshot_unavailable(
        "no snapshot store is in this workspace, so there is nothing to look this id up \
         against. This handler previously echoed the caller's id back with a 64-zero \
         merkle_root and an empty signature, which looks like an addressed response for a \
         snapshot \
         that does not exist",
    )
}

/// The 501 both snapshot reads share, naming what is absent.
fn snapshot_unavailable(detail: &str) -> axum::response::Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetail {
            problem_type: "https://eel.local/problems/not-implemented".into(),
            title: "Not Implemented".into(),
            status: 501,
            detail: detail.into(),
        }),
    )
        .into_response()
}

async fn snapshots_bulk() -> axum::response::Response {
    // A bulk snapshot download is a pre-signed MinIO URL with a short TTL
    // (ADR-0011 Tier 4), so it has to be generated per request against a bucket
    // that exists. Object storage is not in this workspace.
    //
    // This redirected to `https://minio.eel.internal/portal-snapshots/latest.tar.gz`
    // -- a hardcoded path on a host that does not resolve. A 302 is a successful
    // answer: a client follows it and gets a DNS failure, having been told the
    // snapshot was ready to download. The old comment said "Stub returns 302",
    // which is the honest description of a fabrication.
    // Built inline rather than via a helper: `ProblemDetail` in this crate has
    // only not_found and bad_request constructors, and neither is the right status
    // for "this process cannot issue the URL at all".
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ProblemDetail {
            problem_type: "https://eel.local/problems/not-implemented".into(),
            title: "Not Implemented".into(),
            status: 501,
            detail: "a bulk snapshot download is a pre-signed MinIO URL generated per \
                     request, and object storage is not in this workspace. This handler \
                     previously redirected to a hardcoded path on minio.eel.internal, \
                     which does not resolve: a 302 tells a client the download is ready \
                     and then fails at the host name"
                .into(),
        }),
    )
        .into_response()
}

/// Verify a snapshot against its inclusion proof.
///
/// Previously answered 200 with a proof built entirely from placeholders: a
/// 64-zero `leaf_hash`, a 64-zero `root`, `siblings: []`, `tree_size: 1` and
/// `sth_sig: ""`. So a client asking "is this snapshot in the log" was handed a
/// proof against a zero root with an empty sibling path — which recomputes to
/// nothing and verifies nothing, while `verified: false` sits there looking like a
/// field to be trusted rather than an absence to be noticed.
async fn snapshot_verify(Path(id): Path<String>) -> axum::response::Response {
    let Ok(uid) = id.parse::<Uuid>() else {
        return ProblemDetail::bad_request("snapshot_id must be a valid UUID").into_response();
    };
    let _ = uid;
    snapshot_unavailable(
        "verifying a snapshot means recomputing its Merkle root from the leaf and its \
         sibling path and comparing that against a signed tree head. The sibling path is \
         not published here and there is no transparency log to check against. This handler \
         previously returned 200 with a 64-zero leaf_hash, a 64-zero root, no siblings and an \
         empty sth_sig -- a proof against a zero root that verifies nothing",
    )
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
        .layer(axum::middleware::from_fn(nosniff))
}
// `X-Content-Type-Options: nosniff` on every response, not just snapshot ones.
async fn nosniff(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let response = next.run(request).await;
    use axum::http::header::HeaderValue;
    let (mut parts, body) = response.into_parts();
    parts.headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    axum::response::Response::from_parts(parts, body)
}

