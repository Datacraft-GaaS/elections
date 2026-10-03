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

//! Public `/verify/v1/*` handlers.
//!
//! Every handler here reads or advances [`crate::verify_store::VerifyStore`].
//! The one that mattered most was `verify_token`, which returned
//! `included: true` for **any** token with a proof reference of
//! `proof-<query_id>`. That is the endpoint a member of the public uses to check
//! whether their token is in the published results, and it said yes to everything.
//!
//! Nothing in this workspace publishes results, so a token cannot honestly be
//! *included*. What can be answered is whether the token was recorded here, which
//! is a different question — so the response names which question it is answering
//! and carries the tree the answer comes from.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

use crate::models::{TokenVerifyRequest, TokenVerifyResponse};
use crate::verify_store::SharedVerifyStore;

/// A problem+json response.
fn problem(status: StatusCode, detail: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail.into(),
        })),
    )
        .into_response()
}

use axum::response::IntoResponse;

/// A verified token: a token that is in the recorded set, and whose inclusion
/// proof verifies against the root the service reports.
///
/// `POST /verify/v1/token/record` is what puts a token in the set; it exists so
/// the verify surface is testable end to end and so a token recorded by one part
/// of the system can be checked by another.
pub async fn record_token(
    State(store): State<SharedVerifyStore>,
    Json(req): Json<TokenVerifyRequest>,
) -> axum::response::Response {
    info!(token_sha256 = %req.token_sha256, "verify.record_token");
    if req.token_sha256.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "token_sha256 is required");
    }
    let record = store.record_token(req.token_sha256.trim(), None, "verify/v1/token/record");
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "token_sha256": record.token_sha256,
            "leaf_hash": record.leaf_hash,
            "leaf_index": record.leaf_index,
            "recorded_at": record.recorded_at,
        })),
    )
        .into_response()
}

/// Check whether a token was recorded here, with a proof that verifies.
pub async fn verify_token(
    State(store): State<SharedVerifyStore>,
    Json(req): Json<TokenVerifyRequest>,
) -> axum::response::Response {
    let log_query_id = Uuid::now_v7().to_string();
    info!(token_sha256 = %req.token_sha256, "verify.verify_token");
    let sha = req.token_sha256.trim();
    if sha.is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "token_sha256 is required");
    }
    match store.proof_for(sha) {
        Some((proof, root, leaf_index, tree_size)) => {
            // The proof is only reported because it was verified. `included` is
            // scoped to *this* set and says so in the field beside it.
            Json(serde_json::json!({
                "query_id": log_query_id,
                "token_sha256": sha,
                "included_in_this_set": true,
                "inclusion_proof": proof,
                "merkle_root": root,
                "leaf_index": leaf_index,
                "tree_size": tree_size,
                "scope": "tokens recorded through this BFF",
                "tlog_inclusion_proven": false,
                "verified_at": Utc::now(),
            }))
            .into_response()
        }
        None => {
            // A token that was never recorded is not included. The old handler
            // answered `included: true` here, which is the fabrication.
            Json(serde_json::json!({
                "query_id": log_query_id,
                "token_sha256": sha,
                "included_in_this_set": false,
                "inclusion_proof": Value::Null,
                "merkle_root": Value::Null,
                "scope": "tokens recorded through this BFF",
                "tlog_inclusion_proven": false,
                "verified_at": Utc::now(),
            }))
            .into_response()
        }
    }
}

/// The recorded token set, and the root over it.
pub async fn get_results(State(store): State<SharedVerifyStore>) -> axum::response::Response {
    let tokens = store.tokens();
    let tree = store.tree();
    // Previously `data: []` with a note about a relay from a service that is not in
    // this workspace. This is the set this process actually holds, and an empty one
    // is reported as empty rather than dressed up as a successful relay.
    Json(serde_json::json!({
        "status": if tokens.is_empty() { "empty" } else { "ok" },
        "scope": "tokens recorded through this BFF",
        "count": tokens.len(),
        "data": tokens.iter().map(|t| serde_json::json!({
            "token_sha256": t.token_sha256,
            "constituency_code": t.constituency_code,
            "source": t.source,
            "leaf_index": t.leaf_index,
            "recorded_at": t.recorded_at,
        })).collect::<Vec<Value>>(),
        "merkle_root": tree.as_ref().map(|t| hex::encode(t.root())),
        "tlog_inclusion_proven": false,
    }))
    .into_response()
}

/// The recorded tokens for one constituency, or the whole country.
pub async fn get_results_by_scope(
    State(store): State<SharedVerifyStore>,
    Path((scope, code)): Path<(String, String)>,
) -> axum::response::Response {
    info!(scope, code, "verify.get_results_by_scope");
    let tokens = match scope.as_str() {
        "constituency" => store.tokens_for_constituency(&code),
        "national" => store.tokens(),
        other => {
            return problem(
                StatusCode::NOT_FOUND,
                format!("unknown scope {other}; this service holds constituency and national sets"),
            )
        }
    };
    if tokens.is_empty() && scope == "constituency" {
        // Previously `data: null` with a note, for any scope and code.
        return problem(
            StatusCode::NOT_FOUND,
            format!("no tokens have been recorded for constituency {code}"),
        );
    }
    Json(serde_json::json!({
        "scope": scope,
        "code": code,
        "count": tokens.len(),
        "data": tokens.iter().map(|t| serde_json::json!({
            "token_sha256": t.token_sha256,
            "leaf_index": t.leaf_index,
            "recorded_at": t.recorded_at,
        })).collect::<Vec<Value>>(),
        "tlog_inclusion_proven": false,
    }))
    .into_response()
}

/// The inclusion proof for a recorded token.
pub async fn get_token_proof(
    State(store): State<SharedVerifyStore>,
    Path(sha): Path<String>,
) -> axum::response::Response {
    info!(sha, "verify.get_token_proof");
    // Previously `"proof": null` with a note, for any sha.
    match store.proof_for(sha.trim()) {
        Some((proof, root, leaf_index, tree_size)) => Json(serde_json::json!({
            "sha": sha.trim(),
            "proof": proof,
            "merkle_root": root,
            "leaf_index": leaf_index,
            "tree_size": tree_size,
            "tlog_inclusion_proven": false,
        }))
        .into_response(),
        None => problem(
            StatusCode::NOT_FOUND,
            format!("no token with that digest has been recorded here"),
        ),
    }
}

/// Anomaly flags derived from the queue state.
pub async fn get_anomalies(State(store): State<SharedVerifyStore>) -> impl IntoResponse {
    // Previously `anomalies: []` for any query, which reads as "nothing detected".
    let anomalies = store.anomalies();
    Json(serde_json::json!({
        "count": anomalies.len(),
        "anomalies": anomalies,
        "derived_from": "per-station check-in history",
    }))
}

/// Check an agent into a station's queue, or out of it.
pub async fn queue_check_in(
    State(store): State<SharedVerifyStore>,
    Path(station): Path<String>,
    Json(req): Json<Value>,
) -> axum::response::Response {
    let agent = req
        .get("agent_id_hash")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    if agent.is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "agent_id_hash is required",
        );
    }
    let (queue, duplicate) = store.check_in(&station, &agent);
    Json(serde_json::json!({
        "station": station,
        "checked_in": queue.checked_in.len(),
        "queue": queue.checked_in,
        "duplicate_check_in": duplicate,
    }))
    .into_response()
}

pub async fn queue_check_out(
    State(store): State<SharedVerifyStore>,
    Path(station): Path<String>,
    Json(req): Json<Value>,
) -> axum::response::Response {
    let agent = req
        .get("agent_id_hash")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    if agent.is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "agent_id_hash is required",
        );
    }
    match store.check_out(&station, &agent) {
        Some(queue) => Json(serde_json::json!({
            "station": station,
            "checked_in": queue.checked_in.len(),
            "queue": queue.checked_in,
        }))
        .into_response(),
        None => problem(
            StatusCode::NOT_FOUND,
            format!("no queue has been opened for station {station}"),
        ),
    }
}

/// A station's queue, as it currently stands.
pub async fn get_queue_snapshot(
    State(store): State<SharedVerifyStore>,
    Path(station): Path<String>,
) -> axum::response::Response {
    // Previously `queue_length: 0` for any station, which reads as "no one is
    // waiting" for every station in the country.
    match store.queue(&station) {
        Some(queue) => Json(serde_json::json!({
            "station": station,
            "queue_length": queue.checked_in.len(),
            "queue": queue.checked_in,
            "total_check_ins": queue.history.len(),
        }))
        .into_response(),
        None => problem(
            StatusCode::NOT_FOUND,
            format!("no queue has been opened for station {station}"),
        ),
    }
}
