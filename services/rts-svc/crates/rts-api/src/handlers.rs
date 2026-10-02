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

//! HTTP handlers for the RTS surface.
//!
//! Every handler here reads or advances [`crate::store::AppState`]. The two that
//! mattered most previously were:
//!
//! - `get_form_proof` returned `merkle_root: "0".repeat(64)` — the *zero root*,
//!   which is not a Merkle root of anything — together with an empty proof. A
//!   client that verified the proof would reject it; a client that only checked
//!   "is a root present" would accept it. `packages/eel-crypto-rs` has a real
//!   `MerkleTree` and it is used here.
//! - `get_icr_job` minted a **new** `job_id` on every read, so a client polling
//!   one form's ICR job got a different id each time and could never correlate
//!   two polls.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

use crate::models::*;
use crate::store::{CompilationJob, SharedState, StoredDispute, StoredForm, StoredToken};

/// A problem+json response.
pub fn problem(status: StatusCode, detail: impl Into<String>) -> axum::response::Response {
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

/// How long a provisioned device token is valid for.
const TOKEN_TTL_HOURS: i64 = 24;

/// The Form 34A arithmetic check, run on receipt.
///
/// Returns an error message when the form is internally inconsistent, because a
/// Form 34A whose own numbers disagree is evidence of a transcription fault and
/// should be rejected rather than consolidated.
fn check_form34a(req: &Form34ARequest) -> Option<String> {
    if req.station_code.trim().is_empty()
        || req.constituency_code.trim().is_empty()
        || req.county_code.trim().is_empty()
    {
        return Some("station_code, constituency_code and county_code are required".to_string());
    }
    // rejected + valid + spoilt must reconcile against votes_cast.
    let accounted = req.valid_votes + req.rejected_votes + req.spoilt_ballots;
    if req.votes_cast != accounted {
        return Some(format!(
            "votes_cast ({}) does not equal valid ({}) + rejected ({}) + spoilt ({}) = {accounted}",
            req.votes_cast, req.valid_votes, req.rejected_votes, req.spoilt_ballots
        ));
    }
    if req.votes_cast > req.registered_voters {
        return Some(format!(
            "votes_cast ({}) exceeds registered_voters ({})",
            req.votes_cast, req.registered_voters
        ));
    }
    let candidate_total: u64 = req.candidates.iter().map(|c| c.votes as u64).sum();
    if candidate_total != req.valid_votes as u64 {
        return Some(format!(
            "candidate votes total ({candidate_total}) does not equal valid_votes ({})",
            req.valid_votes
        ));
    }
    None
}

/// The acknowledgement record for a stored form.
fn accepted(form: &StoredForm, status: &str) -> FormAccepted {
    FormAccepted {
        form_id: form.form_id,
        status: status.to_string(),
        icr_job_id: form.icr.as_ref().map(|j| j.job_id.clone()),
    }
}

pub async fn submit_form(
    State(state): State<SharedState>,
    Json(req): Json<Form34ARequest>,
) -> axum::response::Response {
    info!(station_code = %req.station_code, "submit_form: received");
    if let Some(detail) = check_form34a(&req) {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, detail);
    }
    let form = state.record_form(req, false);
    // The scan id is derived from the form, so the acknowledgement endpoint is
    // reachable: previously get_ack answered "pending" for any id forever.
    let scan_id = scan_id_for(&form.form_id);
    state.acknowledge(scan_id, form.form_id);
    (StatusCode::ACCEPTED, Json(accepted(&form, "accepted"))).into_response()
}

pub async fn sms_fallback(
    State(state): State<SharedState>,
    Json(req): Json<SmsFallbackRequest>,
) -> axum::response::Response {
    info!(device_id = %req.device_id, "sms_fallback: received");
    if req.encoded_payload.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "encoded_payload is required",
        );
    }
    // The SMS payload is an encoded envelope this service does not parse. The
    // sequence number is recorded so a replay is detectable, and the payload is
    // stored verbatim rather than being turned into a form: fabricating a form
    // from an unparsed payload is the failure this whole effort is about.
    let envelope = serde_json::json!({
        "device_id": req.device_id,
        "sequence_number": req.sequence_number,
        "encoded_payload": req.encoded_payload,
        "received_via": "sms-fallback",
    });
    let event = state.record_event(envelope);
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "received",
            "event_id": event.event_id,
            "sequence_number": req.sequence_number,
            "blocking_reason": "the SMS payload is an encoded envelope this service does not \
                                parse; it is stored verbatim rather than being turned into a form",
        })),
    )
        .into_response()
}

pub async fn provision_token(
    State(state): State<SharedState>,
    Json(req): Json<TokenRequest>,
) -> axum::response::Response {
    info!(device_id = %req.device_id, "provision_token");
    if req.device_id.trim().is_empty() || req.svid.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "device_id and svid are required",
        );
    }
    let issued_at = chrono::Utc::now();
    let expires_at = issued_at + chrono::Duration::hours(TOKEN_TTL_HOURS);
    // A real digest over the device id, the svid and the issue time, so the
    // token is bound to this request. It is a capability token, not a
    // signature: no signing key is reachable from this process.
    let token = format!(
        "rts-token-{}",
        digest(&[&req.device_id, &req.svid, &issued_at.to_rfc3339()])
    );
    state.record_token(StoredToken {
        device_id: req.device_id.clone(),
        token: token.clone(),
        issued_at: issued_at.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
    });
    (
        StatusCode::OK,
        Json(TokenResponse {
            token,
            expires_at: expires_at.to_rfc3339(),
        }),
    )
        .into_response()
}

/// The acknowledgement scan id for a form.
///
/// Derived from the form id rather than minted, so the acknowledgement endpoint
/// is reachable and the same form always yields the same scan id.
fn scan_id_for(form_id: &Uuid) -> Uuid {
    Uuid::new_v5(form_id, b"eel.rts.form34a.scan.v1")
}

/// The ICR job id for a form. Stable across reads, unlike a freshly minted one.
fn icr_job_id_for(form_id: &Uuid) -> String {
    Uuid::new_v5(form_id, b"eel.rts.icr.job.v1").to_string()
}

/// A snapshot id derived from the tree root, so a changed root means a new
/// snapshot.
fn snapshot_id_for(tree: &eel_crypto_rs::merkle::MerkleTree) -> Uuid {
    let root = tree.root();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&root[..16]);
    Uuid::new_v5(&Uuid::from_bytes(bytes), b"eel.rts.snapshot.v1")
}

/// A digest over the given parts, `sha256:`-prefixed.
fn digest(parts: &[&str]) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    for p in parts {
        h.update(p.as_bytes());
        h.update([0x1e]);
    }
    format!("sha256:{}", hex::encode(h.finalize()))
}

pub async fn ingest_event(
    State(state): State<SharedState>,
    Json(req): Json<EventIngestRequest>,
) -> axum::response::Response {
    // The envelope must carry an id and a type; an envelope with neither cannot be
    // attributed to anything and is not worth storing.
    let has_id = req
        .envelope
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let has_type = req
        .envelope
        .get("type")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if !has_id || !has_type {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "the envelope needs a non-empty `id` and `type`",
        );
    }
    let event = state.record_event(req.envelope);
    (
        StatusCode::ACCEPTED,
        Json(EventIngestResponse {
            event_id: event.event_id,
            status: "accepted".to_string(),
        }),
    )
        .into_response()
}

pub async fn get_ack(
    State(state): State<SharedState>,
    Path(scan_id): Path<Uuid>,
) -> axum::response::Response {
    // Previously any scan id returned `status: "pending", form_id: None`, so a
    // form that had been submitted and scanned read as never received.
    match state.acknowledged(&scan_id) {
        Some(form_id) => Json(AckResponse {
            scan_id,
            status: "received".to_string(),
            form_id: Some(form_id),
        })
        .into_response(),
        None => problem(
            StatusCode::NOT_FOUND,
            format!("no scan with id {scan_id} has been acknowledged"),
        ),
    }
}

pub async fn list_snapshots(State(state): State<SharedState>) -> impl IntoResponse {
    // A snapshot exists for every constituency that has at least one form, and
    // carries the Merkle root over the forms received so far.
    let all_forms = state.forms();
    let mut seen: Vec<String> = Vec::new();
    for form in &all_forms {
        if !seen.contains(&form.constituency_code) {
            seen.push(form.constituency_code.clone());
        }
    }
    seen.sort();
    let tree = state.merkle();
    let snapshots: Vec<Value> = seen
        .iter()
        .map(|code| {
            serde_json::json!({
                "scope": "constituency",
                "code": code,
                "forms": all_forms.iter().filter(|f| &f.constituency_code == code).count(),
                "merkle_root": tree.as_ref().map(|t| hex::encode(t.root())),
            })
        })
        .collect();
    Json(serde_json::json!({
        "snapshots": snapshots,
        "tlog_inclusion_proven": false,
    }))
}

pub async fn get_snapshot(
    State(state): State<SharedState>,
    Path((scope, code)): Path<(String, String)>,
) -> axum::response::Response {
    // Previously any scope/code returned a response with every field null.
    if scope != "constituency" {
        return problem(
            StatusCode::NOT_FOUND,
            format!("unknown snapshot scope {scope}; this service consolidates by constituency"),
        );
    }
    let Some(form34c) = state.consolidated_34c(&code) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no Form 34A has been received for constituency {code}"),
        );
    };
    let tree = state.merkle();
    Json(SnapshotResponse {
        scope,
        code,
        snapshot_id: tree.as_ref().map(|t| snapshot_id_for(t)),
        published_at: None,
        form34c: Some(serde_json::to_value(form34c).unwrap_or(Value::Null)),
        merkle_root: tree.as_ref().map(|t| hex::encode(t.root())),
    })
    .into_response()
}

pub async fn get_form_proof(
    State(state): State<SharedState>,
    Path(form_id): Path<Uuid>,
) -> axum::response::Response {
    // This previously returned `merkle_root: "0".repeat(64)` -- the zero root --
    // with an empty proof. Now the root and the proof come from a real
    // `MerkleTree` over the forms this process has received, and the proof is
    // verified here before it is returned, so a proof that would not verify is
    // never served.
    let Some(form) = state.form(&form_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no Form 34A with id {form_id} has been received"),
        );
    };
    let Some(tree) = state.merkle() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "no Form 34A has been received, so there is no tree to prove against",
        );
    };
    let Ok(proof) = tree.inclusion_proof(form.leaf_index) else {
        return problem(
            StatusCode::INTERNAL_SERVER_ERROR,
            "the form's leaf index is outside the tree",
        );
    };
    // Do not serve a proof that does not verify.
    if tree
        .verify_inclusion(form.leaf_index, form.leaf_hash.as_bytes(), &proof)
        .is_err()
    {
        return problem(
            StatusCode::INTERNAL_SERVER_ERROR,
            "the computed inclusion proof does not verify against the root",
        );
    }
    let root = hex::encode(tree.root());
    Json(serde_json::json!({
        "form_id": form.form_id,
        "merkle_proof": proof.iter().map(hex::encode).collect::<Vec<String>>(),
        "merkle_root": root,
        "leaf_hash": form.leaf_hash,
        "leaf_index": form.leaf_index,
        "tree_size": tree.size(),
        // A real root over this process's forms, and a proof that verifies
        // against it. Not a transparency-log inclusion proof: there is no
        // append-only log here and no cross-process agreement on the leaf set.
        "tlog_inclusion_proven": false,
    }))
    .into_response()
}

async fn compile_form(
    state: &SharedState,
    form_kind: &str,
    scope: Option<&str>,
) -> axum::response::Response {
    let job = CompilationJob {
        job_id: Uuid::now_v7().to_string(),
        form_kind: form_kind.to_string(),
        // A queued job is not a finished one. Previously this returned
        // "compilation_queued" for any input and recorded nothing at all.
        status: "queued".to_string(),
        requested_at: chrono::Utc::now().to_rfc3339(),
        blocking_reason: Some(format!(
            "{form_kind} compilation aggregates every station's return for a constituency and \
             seals the result; sealing needs the HSM ceremony, which is not reachable from this \
             process, so the job is recorded as queued rather than reported complete"
        )),
    };
    let _ = scope;
    state.record_job(job.clone());
    (StatusCode::ACCEPTED, Json(serde_json::json!({
        "job_id": job.job_id,
        "form_kind": job.form_kind,
        "status": job.status,
        "requested_at": job.requested_at,
        "blocking_reason": job.blocking_reason,
    })))
    .into_response()
}

pub async fn compile_form34b(
    State(state): State<SharedState>,
    Json(scope): Json<Value>,
) -> axum::response::Response {
    compile_form(&state, "34B", scope.get("scope").and_then(|v| v.as_str())).await
}

pub async fn compile_form34c(
    State(state): State<SharedState>,
    Json(scope): Json<Value>,
) -> axum::response::Response {
    compile_form(&state, "34C", scope.get("scope").and_then(|v| v.as_str())).await
}

pub async fn get_icr_job(
    State(state): State<SharedState>,
    Path(form_id): Path<Uuid>,
) -> axum::response::Response {
    // Previously this minted a *new* job_id on every read, so a client polling one
    // form's job could never correlate two polls.
    let Some(form) = state.form(&form_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no Form 34A with id {form_id} has been received"),
        );
    };
    match &form.icr {
        Some(job) => Json(job.clone()).into_response(),
        None => {
            // The job id is derived from the form, so it is stable across reads.
            let job = IcrJobResult {
                job_id: icr_job_id_for(&form.form_id),
                form_id,
                fields: Vec::new(),
                status: "pending".to_string(),
            };
            Json(job).into_response()
        }
    }
}

pub async fn get_icr_result(
    State(state): State<SharedState>,
    Path(job_id): Path<String>,
) -> axum::response::Response {
    // Previously any job id answered `pending` with no fields, forever.
    let parsed = Uuid::parse_str(&job_id).ok();
    let job = parsed
        .and_then(|_| {
            state
                .forms()
                .into_iter()
                .find_map(|f| f.icr.filter(|j| j.job_id == job_id))
        });
    match job {
        Some(j) => Json(serde_json::json!({
            "job_id": j.job_id,
            "form_id": j.form_id,
            "status": j.status,
            "fields": j.fields,
        }))
        .into_response(),
        None => problem(
            StatusCode::NOT_FOUND,
            format!("no ICR result has been produced for job {job_id}"),
        ),
    }
}

pub async fn get_audit(
    State(state): State<SharedState>,
    Path(form_id): Path<Uuid>,
) -> axum::response::Response {
    let Some(form) = state.form(&form_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no Form 34A with id {form_id} has been received"),
        );
    };
    let scan_id = scan_id_for(&form.form_id);
    Json(serde_json::json!({
        "form_id": form.form_id,
        "station_code": form.station_code,
        "constituency_code": form.constituency_code,
        "received_at": form.received_at,
        "leaf_index": form.leaf_index,
        "leaf_hash": form.leaf_hash,
        "scan_id": scan_id,
        "via_sms": form.via_sms,
        "events": [{
            "event_id": form.form_id.to_string(),
            "event_type": "eel.rts.form34a.received.v1",
            "recorded_at": form.received_at,
        }],
    }))
    .into_response()
}

pub async fn create_dispute(
    State(state): State<SharedState>,
    Json(req): Json<DisputeRequest>,
) -> axum::response::Response {
    info!(form_id = %req.form_id, "create_dispute");
    if state.form(&req.form_id).is_none() {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no Form 34A with id {} has been received", req.form_id),
        );
    }
    if req.reason.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "reason is required");
    }
    let dispute = DisputeCreated {
        dispute_id: Uuid::now_v7(),
        status: "open".to_string(),
    };
    state.record_dispute(StoredDispute {
        dispute: dispute.clone(),
        form_id: req.form_id,
        reason: req.reason,
        ro_id: req.ro_id,
        resolution: None,
        resolved_at: None,
    });
    (StatusCode::CREATED, Json(dispute)).into_response()
}

pub async fn resolve_dispute(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(req): Json<DisputeResolveRequest>,
) -> axum::response::Response {
    // Previously this answered "resolved" for any dispute id and never looked at
    // the countersign_token the request carries.
    let Some(dispute) = state.dispute(&id) else {
        return problem(StatusCode::NOT_FOUND, format!("no dispute with id {id}"));
    };
    if dispute.resolution.is_some() {
        return problem(
            StatusCode::CONFLICT,
            format!("dispute {id} has already been resolved"),
        );
    }
    // The countersign token must be a token this service issued, for the running
    // officer named on the dispute. That is what makes the countersignature mean
    // something: the same token cannot countersign a different dispute.
    let Some(token) = state.token(&req.countersign_token) else {
        return problem(
            StatusCode::FORBIDDEN,
            "countersign_token is not a token this service issued",
        );
    };
    if token.device_id != dispute.ro_id {
        return problem(
            StatusCode::FORBIDDEN,
            format!(
                "countersign_token was issued to {} but the dispute is under {}",
                token.device_id,
                dispute.ro_id
            ),
        );
    }
    if req.resolution.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "resolution is required");
    }
    let resolved = DisputeResolved {
        dispute_id: id,
        status: "resolved".to_string(),
    };
    let resolved_at = chrono::Utc::now().to_rfc3339();
    state.resolve_dispute(&id, &req.resolution);
    Json(serde_json::json!({
        "dispute_id": resolved.dispute_id,
        "status": resolved.status,
        "resolution": req.resolution,
        "resolved_at": resolved_at,
    }))
    .into_response()
}

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}
