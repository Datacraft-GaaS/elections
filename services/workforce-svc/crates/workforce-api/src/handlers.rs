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

//! HTTP handlers for workforce-svc. All stateful handlers take `State<Store>`
//! and map [`StoreError`] to problem responses (404/409/500); request
//! validation failures return 422.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::domain::*;
use crate::events::*;
use crate::store::{Store, StoreError};
use eel_outbox_rs::OutboxEnvelope;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn problem(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "error": msg.into() }))).into_response()
}

fn store_error_response(err: StoreError) -> Response {
    match err {
        StoreError::NotFound => problem(StatusCode::NOT_FOUND, "not found"),
        StoreError::OpenClaimExists => problem(StatusCode::CONFLICT, err.to_string()),
        other => problem(StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
    }
}

/// Returns the 422 problem response when `value` is not a 64-hex hash.
fn hash_problem(value: &str, what: &str) -> Option<Response> {
    if is_hex64(value) {
        None
    } else {
        Some(problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("{what} must be a 64-character hex hash"),
        ))
    }
}

/// Per-(county, role) selection capacity; configurable via env, default 1000.
fn waitlist_capacity() -> i64 {
    std::env::var("WORKFORCE_WAITLIST_CAPACITY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1_000)
}

/// Records a worker notification, resolving the delivery target from the
/// worker's preferred contact point when `channel` is None. Best-effort: a
/// missing worker or contact is advisory, never a failure.
async fn record_notification(
    store: &Store,
    worker_id: Uuid,
    kind: NotificationKind,
    subject: impl Into<String>,
    body: impl Into<String>,
) -> Option<Notification> {
    let delivery = store.preferred_contact(worker_id).await.ok().flatten();
    let notification = Notification {
        notification_id: Uuid::now_v7(),
        worker_id,
        kind,
        channel: delivery.as_ref().map(|c| c.channel),
        subject: subject.into(),
        body: body.into(),
        delivery_ref: delivery.map(|c| c.value_hash_hex),
        created_at: Utc::now(),
    };
    match store.insert_notification(&notification).await {
        Ok(()) => Some(notification),
        Err(_) => None,
    }
}

/// Best-effort cross-service event emission (design §8). A failed emit is
/// logged, never surfaced to the caller as a failed business operation.
async fn emit_event(store: &Store, envelope: &OutboxEnvelope) {
    if let Err(e) = store.emit(envelope).await {
        tracing::warn!(error = %e, topic = %envelope.topic, "outbox emit failed");
    }
}

pub(crate) async fn health() -> impl IntoResponse {
    StatusCode::OK
}

// ---------------------------------------------------------------------------
// Registry & contacts
// ---------------------------------------------------------------------------

pub async fn create_worker(
    State(store): State<Store>,
    Json(req): Json<CreateWorkerRequest>,
) -> Response {
    info!(home_county = %req.home_county, "create_worker");
    if let Some(resp) = hash_problem(&req.national_id_hash, "national_id_hash") {
        return resp;
    }
    if let Some(nok) = &req.next_of_kin {
        if let Some(resp) = hash_problem(&nok.phone_hash, "next_of_kin.phone_hash") {
            return resp;
        }
    }
    let worker = Worker {
        worker_id: Uuid::now_v7(),
        legal_name: req.legal_name,
        national_id_hash: req.national_id_hash,
        home_county: req.home_county,
        role_eligibility: req.role_eligibility,
        payroll_contract_ref: req.payroll_contract_ref,
        next_of_kin_name: req.next_of_kin.as_ref().map(|n| n.name.clone()),
        next_of_kin_phone_hash: req.next_of_kin.as_ref().map(|n| n.phone_hash.clone()),
        created_at: Utc::now(),
    };
    match store.insert_worker(&worker).await {
        Ok(()) => (StatusCode::CREATED, Json(worker)).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn get_worker(State(store): State<Store>, Path(id): Path<Uuid>) -> Response {
    match store.get_worker(id).await {
        Ok(Some(w)) => Json(w).into_response(),
        Ok(None) => problem(StatusCode::NOT_FOUND, "worker not found"),
        Err(e) => store_error_response(e),
    }
}

/// GET /v1/workforce/workers/{id}/badge — a verifiable appointment credential
/// (QR-friendly) that the field app renders as a scannable badge.
pub async fn get_badge(State(store): State<Store>, Path(id): Path<Uuid>) -> Response {
    let worker = match store.get_worker(id).await {
        Ok(Some(w)) => w,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "worker not found"),
        Err(e) => return store_error_response(e),
    };
    let posting = match store.active_posting_for_worker(id).await {
        Ok(p) => p,
        Err(e) => return store_error_response(e),
    };
    let role = posting
        .as_ref()
        .map(|p| p.role_code.clone())
        .or_else(|| worker.role_eligibility.first().cloned())
        .unwrap_or_default();
    let station_code = posting.as_ref().map(|p| p.station_code.clone());
    let issued_at = Utc::now();
    let badge_hash = compute_badge_hash(
        worker.worker_id,
        &worker.legal_name,
        &role,
        station_code.as_deref(),
        worker.payroll_contract_ref.as_deref(),
    );
    Json(WorkerBadge {
        worker_id: worker.worker_id,
        name: worker.legal_name,
        role,
        station_code,
        contract_ref: worker.payroll_contract_ref,
        issued_at,
        badge_hash,
    })
    .into_response()
}

/// Deterministic SHA-256 over the credential content, so a scanned badge can
/// be re-verified against the same fields without storing extra secrets.
fn compute_badge_hash(
    worker_id: Uuid,
    name: &str,
    role: &str,
    station_code: Option<&str>,
    contract_ref: Option<&str>,
) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(worker_id.to_string().as_bytes());
    hasher.update(name.as_bytes());
    hasher.update(role.as_bytes());
    hasher.update(station_code.unwrap_or("").as_bytes());
    hasher.update(contract_ref.unwrap_or("").as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn list_workers(
    State(store): State<Store>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let role = params.get("role").cloned();
    match store
        .list_workers(
            params.get("home_county").map(String::as_str),
            role.as_deref(),
        )
        .await
    {
        Ok(workers) => Json(serde_json::json!({ "workers": workers })).into_response(),
        Err(e) => store_error_response(e),
    }
}

/// Pick the contact point a channel-scoped action applies to: exact value-hash
/// match first, then the preferred point, then the most recently changed.
fn resolve_contact(contacts: &[ContactPoint], value_hash_hex: &str) -> Option<ContactPoint> {
    if let Some(c) = contacts.iter().find(|c| c.value_hash_hex == value_hash_hex) {
        return Some(c.clone());
    }
    if let Some(c) = contacts.iter().find(|c| c.preferred) {
        return Some(c.clone());
    }
    contacts.iter().max_by_key(|c| c.changed_at).cloned()
}

pub async fn upsert_contact(
    State(store): State<Store>,
    Path((worker_id, channel_path)): Path<(Uuid, String)>,
    Json(req): Json<UpsertContactRequest>,
) -> Response {
    let Some(channel) = Channel::parse(&channel_path) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("unknown contact channel '{channel_path}'"),
        );
    };
    info!(worker_id = %worker_id, channel = channel.as_str(), remove = req.remove, "upsert_contact");
    if store.get_worker(worker_id).await.ok().flatten().is_none() {
        return problem(StatusCode::NOT_FOUND, "worker not found");
    }

    // Removal is reason-gated (audit requirement).
    if req.remove {
        let Some(reason) = req
            .reason
            .as_deref()
            .map(str::trim)
            .filter(|r| !r.is_empty())
        else {
            return problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                "removing a contact point requires a non-empty reason",
            );
        };
        let contacts = match store.list_contacts(worker_id, Some(channel)).await {
            Ok(c) => c,
            Err(e) => return store_error_response(e),
        };
        let Some(target) = resolve_contact(&contacts, &req.value_hash_hex) else {
            return problem(StatusCode::NOT_FOUND, "no contact point for channel");
        };
        let at = Utc::now();
        if let Err(e) = store.remove_contact_by_id(target.contact_id).await {
            return store_error_response(e);
        }
        let entry = ContactLogEntry {
            worker_id,
            channel,
            at,
            actor: req.actor.clone(),
            action: ContactAction::Removed,
            reason: Some(reason.to_string()),
        };
        if let Err(e) = store.append_contact_log(&entry).await {
            return store_error_response(e);
        }
        return Json(serde_json::json!({ "removed": true, "contact_id": target.contact_id }))
            .into_response();
    }

    if let Some(resp) = hash_problem(&req.value_hash_hex, "value_hash_hex") {
        return resp;
    }
    let at = Utc::now();
    let upserted = store
        .upsert_contact(
            worker_id,
            channel,
            &req.value_hash_hex,
            req.verified,
            req.consented,
            req.preferred,
            at,
        )
        .await;
    match upserted {
        Ok((point, added, prev_consented)) => {
            let action = if added {
                ContactAction::Added
            } else {
                ContactAction::Updated
            };
            let entry = ContactLogEntry {
                worker_id,
                channel,
                at,
                actor: req.actor.clone(),
                action: action.clone(),
                reason: req.reason.clone(),
            };
            if let Err(e) = store.append_contact_log(&entry).await {
                return store_error_response(e);
            }
            if req.consented && !prev_consented {
                let entry = ContactLogEntry {
                    worker_id,
                    channel,
                    at,
                    actor: req.actor.clone(),
                    action: ContactAction::ConsentGranted,
                    reason: None,
                };
                if let Err(e) = store.append_contact_log(&entry).await {
                    return store_error_response(e);
                }
            }
            let status = if added {
                StatusCode::CREATED
            } else {
                StatusCode::OK
            };
            (status, Json(point)).into_response()
        }
        Err(e) => store_error_response(e),
    }
}

pub async fn verify_contact(
    State(store): State<Store>,
    Path((worker_id, channel_path)): Path<(Uuid, String)>,
) -> Response {
    let Some(channel) = Channel::parse(&channel_path) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("unknown contact channel '{channel_path}'"),
        );
    };
    let contacts = match store.list_contacts(worker_id, Some(channel)).await {
        Ok(c) => c,
        Err(e) => return store_error_response(e),
    };
    let Some(target) = resolve_contact(&contacts, "") else {
        return problem(StatusCode::NOT_FOUND, "no contact point for channel");
    };
    let at = Utc::now();
    match store.verify_contact_by_id(target.contact_id, at).await {
        Ok(point) => {
            let entry = ContactLogEntry {
                worker_id,
                channel,
                at,
                actor: "system".to_string(),
                action: ContactAction::Verified,
                reason: None,
            };
            if let Err(e) = store.append_contact_log(&entry).await {
                return store_error_response(e);
            }
            Json(point).into_response()
        }
        Err(e) => store_error_response(e),
    }
}

pub async fn list_contacts(State(store): State<Store>, Path(id): Path<Uuid>) -> Response {
    match store.list_contacts(id, None).await {
        Ok(contacts) => Json(serde_json::json!({ "contacts": contacts })).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn contact_history(State(store): State<Store>, Path(id): Path<Uuid>) -> Response {
    match store.contact_history(id).await {
        Ok(entries) => Json(serde_json::json!({ "entries": entries })).into_response(),
        Err(e) => store_error_response(e),
    }
}

// ---------------------------------------------------------------------------
// Recruitment state machine
// ---------------------------------------------------------------------------

pub async fn create_case(
    State(store): State<Store>,
    Json(req): Json<CreateCaseRequest>,
) -> Response {
    info!(application_ref = %req.application_ref, county = %req.county, "create_recruitment_case");
    if let Some(resp) = hash_problem(&req.national_id_hash, "national_id_hash") {
        return resp;
    }
    if req.application_ref.trim().is_empty()
        || req.role_code.trim().is_empty()
        || req.county.trim().is_empty()
    {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "application_ref, role_code and county are required",
        );
    }
    let case = RecruitmentCase {
        case_id: Uuid::now_v7(),
        application_ref: req.application_ref,
        applicant_name: req.applicant_name,
        national_id_hash: req.national_id_hash,
        role_code: req.role_code,
        county: req.county,
        stage: Stage::Applied,
        composite_score: req.composite_score,
        stage_history: Vec::new(),
        created_at: Utc::now(),
    };
    match store.insert_case(&case).await {
        Ok(()) => (StatusCode::CREATED, Json(case)).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn get_case(State(store): State<Store>, Path(id): Path<Uuid>) -> Response {
    match store.get_case(id).await {
        Ok(Some(c)) => Json(c).into_response(),
        Ok(None) => problem(StatusCode::NOT_FOUND, "recruitment case not found"),
        Err(e) => store_error_response(e),
    }
}

pub async fn list_cases(
    State(store): State<Store>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match store
        .list_cases(
            params.get("stage").map(String::as_str),
            params.get("county").map(String::as_str),
        )
        .await
    {
        Ok(cases) => Json(serde_json::json!({ "cases": cases })).into_response(),
        Err(e) => store_error_response(e),
    }
}

/// Pure gate validation for a recruitment transition. Returns the problem to
/// surface, or `None` when the payload satisfies the configured gates. The
/// `rejected` gate is always enforced (not configurable).
pub fn gate_check(
    target: &str,
    gates: &GateConfig,
    req: &TransitionRequest,
) -> Option<(StatusCode, &'static str)> {
    match target {
        "documents_complete" if gates.documents => {
            if !req
                .documents_probe
                .as_ref()
                .map(|p| p.complete)
                .unwrap_or(false)
            {
                return Some((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "documents_complete gate requires body {\"documents_probe\": {\"complete\": true}}",
                ));
            }
        }
        "evaluated" if gates.exam => {
            let ok = req
                .exam_result
                .as_ref()
                .map(|e| e.passed && !e.session_id.trim().is_empty())
                .unwrap_or(false);
            if !ok {
                return Some((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "evaluated gate requires body {\"exam_result\": {\"passed\": true, \"session_id\": \"...\"}}",
                ));
            }
        }
        "appointed" if gates.contract => {
            if req
                .contract_ref
                .as_deref()
                .map(str::trim)
                .filter(|r| !r.is_empty())
                .is_none()
            {
                return Some((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "appointment requires body {\"contract_ref\": \"...\"}",
                ));
            }
        }
        "rejected"
            if req
                .reason
                .as_deref()
                .map(str::trim)
                .filter(|r| !r.is_empty())
                .is_none() =>
        {
            return Some((
                StatusCode::UNPROCESSABLE_ENTITY,
                "rejection requires a structured reason_code in \"reason\"",
            ));
        }
        _ => {}
    }
    None
}

pub async fn transition_case(
    State(store): State<Store>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransitionRequest>,
) -> Response {
    info!(case_id = %id, to = %req.to, actor = %req.actor, "transition_case");
    let Some(case) = store.get_case(id).await.ok().flatten() else {
        return problem(StatusCode::NOT_FOUND, "recruitment case not found");
    };

    let from = case.stage.state().to_string();
    let target = req.to.as_str();
    let allowed = allowed_transitions(&from);
    if !allowed.contains(&target) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": format!("illegal transition from '{from}' to '{target}'"),
                "from": from,
                "to": target,
                "allowed": allowed,
            })),
        )
            .into_response();
    }

    // External gate payloads — design §1/§2/§3 (configurable via GateConfig).
    let gates = GateConfig::from_env();
    if let Some((status, msg)) = gate_check(target, &gates, &req) {
        return problem(status, msg);
    }

    // Appointment creates the worker from case data (design §4).
    let mut final_to = target.to_string();
    let mut waitlist_rank: Option<i64> = None;
    let mut transition_reason = req.reason.clone();
    let mut appointed_worker: Option<Worker> = None;

    if target == "appointed" {
        let worker = Worker {
            worker_id: Uuid::now_v7(),
            legal_name: case.applicant_name.clone(),
            national_id_hash: case.national_id_hash.clone(),
            home_county: case.county.clone(),
            role_eligibility: vec![case.role_code.clone()],
            payroll_contract_ref: req.contract_ref.clone(),
            next_of_kin_name: None,
            next_of_kin_phone_hash: None,
            created_at: Utc::now(),
        };
        if let Err(e) = store.insert_worker(&worker).await {
            return store_error_response(e);
        }
        info!(worker_id = %worker.worker_id, case_id = %id, "worker appointed from recruitment case");
        record_notification(
            &store,
            worker.worker_id,
            NotificationKind::Appointment,
            "Appointment confirmed",
            format!(
                "Your appointment as {} for {} is confirmed under contract {}.",
                case.role_code,
                case.county,
                req.contract_ref.as_deref().unwrap_or_default()
            ),
        )
        .await;
        appointed_worker = Some(worker);
    }

    // Selection beyond the per-(county, role) capacity auto-places onto the
    // waitlist ranked by descending composite score.
    if target == "selected" {
        let capacity = waitlist_capacity();
        let committed = store.count_committed(&case.county, &case.role_code).await;
        let committed = match committed {
            Ok(n) => n,
            Err(e) => return store_error_response(e),
        };
        if committed >= capacity {
            let ahead = store
                .count_waitlist_ahead(&case.county, &case.role_code, case.composite_score)
                .await;
            let ahead = match ahead {
                Ok(n) => n,
                Err(e) => return store_error_response(e),
            };
            final_to = "waitlist".to_string();
            waitlist_rank = Some(ahead + 1);
            transition_reason = Some("selection capacity reached".to_string());
        }
    }
    if target == "waitlist" {
        let ahead = store
            .count_waitlist_ahead(&case.county, &case.role_code, case.composite_score)
            .await;
        let ahead = match ahead {
            Ok(n) => n,
            Err(e) => return store_error_response(e),
        };
        waitlist_rank = Some(ahead + 1);
    }
    let reject_reason = if final_to == "rejected" {
        req.reason.clone()
    } else {
        None
    };

    let change = StageChange {
        from: from.clone(),
        to: final_to.clone(),
        actor: req.actor.clone(),
        at: Utc::now(),
        reason: transition_reason.clone(),
    };
    if let Err(e) = store
        .apply_transition(
            id,
            &final_to,
            reject_reason.as_deref(),
            waitlist_rank,
            &change,
        )
        .await
    {
        return store_error_response(e);
    }

    // Emit cross-service events (design §8).
    emit_event(&store, &stage_changed_envelope(id, &change)).await;
    if let Some(worker) = &appointed_worker {
        let contract_ref = req.contract_ref.as_deref().unwrap_or_default();
        emit_event(&store, &worker_appointed_envelope(worker, contract_ref)).await;
    }

    match store.get_case(id).await {
        Ok(Some(updated)) => Json(updated).into_response(),
        Ok(None) => problem(StatusCode::NOT_FOUND, "recruitment case not found"),
        Err(e) => store_error_response(e),
    }
}

// ---------------------------------------------------------------------------
// Postings & equipment claims
// ---------------------------------------------------------------------------

fn new_posting(
    worker_id: Uuid,
    station_code: String,
    role_code: String,
    effective_from: chrono::DateTime<Utc>,
    effective_to: Option<chrono::DateTime<Utc>>,
    posted_by: String,
) -> PostingOrder {
    PostingOrder {
        posting_id: Uuid::now_v7(),
        worker_id,
        station_code,
        role_code,
        effective_from,
        effective_to,
        posted_by,
        superseded_by: None,
        status: "active".to_string(),
        created_at: Utc::now(),
    }
}

/// Create one posting (superseding any active one), notify the worker and
/// emit the `posting.activated` event. Shared by single and bulk posting.
async fn post_worker(
    store: &Store,
    worker_id: Uuid,
    station_code: String,
    role_code: String,
    effective_from: chrono::DateTime<Utc>,
    effective_to: Option<chrono::DateTime<Utc>>,
    posted_by: String,
) -> Result<PostingOrder, (StatusCode, String)> {
    if store.get_worker(worker_id).await.ok().flatten().is_none() {
        return Err((StatusCode::NOT_FOUND, "worker not found".into()));
    }
    let previous = store
        .active_posting_for_worker(worker_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let posting = new_posting(
        worker_id,
        station_code,
        role_code,
        effective_from,
        effective_to,
        posted_by,
    );
    store
        .insert_posting(&posting, previous.map(|p| p.posting_id))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    record_notification(
        store,
        posting.worker_id,
        NotificationKind::Posting,
        format!("Posting: {}", posting.station_code),
        format!(
            "You are posted to station {} as {} effective {}.",
            posting.station_code,
            posting.role_code,
            posting.effective_from.format("%Y-%m-%d")
        ),
    )
    .await;
    emit_event(store, &posting_activated_envelope(&posting)).await;
    Ok(posting)
}

pub async fn create_posting(
    State(store): State<Store>,
    Json(req): Json<CreatePostingRequest>,
) -> Response {
    info!(worker_id = %req.worker_id, station_code = %req.station_code, "create_posting");
    match post_worker(
        &store,
        req.worker_id,
        req.station_code,
        req.role_code,
        req.effective_from,
        req.effective_to,
        req.posted_by,
    )
    .await
    {
        Ok(posting) => (StatusCode::CREATED, Json(posting)).into_response(),
        Err((status, msg)) => problem(status, msg),
    }
}

#[derive(Debug, Deserialize)]
pub struct BulkPostingItem {
    pub worker_id: Uuid,
    pub station_code: String,
    pub role_code: String,
    pub effective_from: chrono::DateTime<Utc>,
    #[serde(default)]
    pub effective_to: Option<chrono::DateTime<Utc>>,
    pub posted_by: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkPostingRequest {
    pub postings: Vec<BulkPostingItem>,
}

/// POST /v1/workforce/postings/bulk — post many workers in one request.
/// Each item is applied independently; failures are collected, not fatal.
pub async fn bulk_create_postings(
    State(store): State<Store>,
    Json(req): Json<BulkPostingRequest>,
) -> Response {
    let mut created = Vec::new();
    let mut errors = Vec::new();
    for item in req.postings {
        match post_worker(
            &store,
            item.worker_id,
            item.station_code,
            item.role_code,
            item.effective_from,
            item.effective_to,
            item.posted_by,
        )
        .await
        {
            Ok(posting) => created.push(posting),
            Err((status, msg)) => errors.push(serde_json::json!({
                "worker_id": item.worker_id,
                "status": status.as_u16(),
                "error": msg,
            })),
        }
    }
    Json(serde_json::json!({
        "created": created,
        "errors": errors,
    }))
    .into_response()
}

pub async fn supersede_posting(
    State(store): State<Store>,
    Path(id): Path<Uuid>,
    Json(req): Json<SupersedeRequest>,
) -> Response {
    info!(posting_id = %id, new_station_code = %req.new_station_code, "supersede_posting");
    let existing = match store.get_posting(id).await {
        Ok(Some(p)) => p,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "posting not found"),
        Err(e) => return store_error_response(e),
    };
    if existing.status != "active" {
        return problem(StatusCode::CONFLICT, "posting already superseded");
    }
    let successor = new_posting(
        existing.worker_id,
        req.new_station_code,
        req.role_code.unwrap_or(existing.role_code),
        req.effective_from,
        req.effective_to,
        req.posted_by,
    );
    match store.insert_posting(&successor, Some(id)).await {
        Ok(()) => (StatusCode::CREATED, Json(successor)).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn list_postings(
    State(store): State<Store>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let worker_id = params.get("worker_id").and_then(|v| v.parse().ok());
    let station = params.get("station").cloned();
    let active = params.get("active").and_then(|v| v.parse().ok());
    match store
        .list_postings(worker_id, station.as_deref(), active)
        .await
    {
        Ok(postings) => Json(serde_json::json!({ "postings": postings })).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn create_equipment_claim(
    State(store): State<Store>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateClaimRequest>,
) -> Response {
    info!(posting_id = %id, "create_equipment_claim");
    let posting = match store.get_posting(id).await {
        Ok(Some(p)) => p,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "posting not found"),
        Err(e) => return store_error_response(e),
    };
    // Kit claims attach to an active posting only.
    if posting.status != "active" {
        return problem(
            StatusCode::CONFLICT,
            "equipment can only be claimed against an active posting",
        );
    }
    let claim = EquipmentClaim {
        claim_id: Uuid::now_v7(),
        posting_id: id,
        kit_serials: req.kit_serials,
        acknowledged: false,
        returned: false,
        loss_flags: Vec::new(),
        created_at: Utc::now(),
    };
    match store.insert_claim(&claim).await {
        Ok(()) => {
            emit_event(&store, &kit_issued_envelope(&claim)).await;
            (StatusCode::CREATED, Json(claim)).into_response()
        }
        Err(e) => store_error_response(e),
    }
}

pub async fn acknowledge_claim(State(store): State<Store>, Path(claim_id): Path<Uuid>) -> Response {
    info!(claim_id = %claim_id, "acknowledge_claim");
    match store.update_claim(claim_id, Some(true), None).await {
        Ok(claim) => Json(claim).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn return_claim(State(store): State<Store>, Path(claim_id): Path<Uuid>) -> Response {
    info!(claim_id = %claim_id, "return_claim");
    match store.update_claim(claim_id, None, Some(true)).await {
        Ok(claim) => {
            emit_event(&store, &kit_returned_envelope(&claim)).await;
            Json(claim).into_response()
        }
        Err(e) => store_error_response(e),
    }
}

pub async fn flag_loss(
    State(store): State<Store>,
    Path(claim_id): Path<Uuid>,
    Json(req): Json<FlagLossRequest>,
) -> Response {
    info!(claim_id = %claim_id, serials = req.serials.len(), "flag_loss");
    if req.reason.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "loss flag requires a non-empty reason",
        );
    }
    let flags: Vec<String> = req
        .serials
        .iter()
        .map(|s| format!("{s}:{}", req.reason))
        .collect();
    match store.append_loss_flags(claim_id, &flags).await {
        Ok(claim) => Json(claim).into_response(),
        Err(e) => store_error_response(e),
    }
}

// ---------------------------------------------------------------------------
// Tasking
// ---------------------------------------------------------------------------

pub async fn create_template(
    State(store): State<Store>,
    Json(req): Json<CreateTemplateRequest>,
) -> Response {
    info!(role_code = %req.role_code, phase = req.phase.as_str(), "create_task_template");
    let template = TaskTemplate {
        template_id: Uuid::now_v7(),
        role_code: req.role_code,
        phase: req.phase,
        title: req.title,
        critical: req.critical,
        training_content_id: req.training_content_id,
        created_at: Utc::now(),
    };
    match store.insert_template(&template).await {
        Ok(()) => (StatusCode::CREATED, Json(template)).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn list_templates(
    State(store): State<Store>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match store
        .list_templates(
            params.get("role").map(String::as_str),
            params.get("phase").map(String::as_str),
        )
        .await
    {
        Ok(templates) => Json(serde_json::json!({ "templates": templates })).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn assign_task(
    State(store): State<Store>,
    Json(req): Json<AssignTaskRequest>,
) -> Response {
    info!(posting_id = %req.posting_id, template_id = %req.template_id, "assign_task");
    let template = match store.get_template(req.template_id).await {
        Ok(Some(t)) => t,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "task template not found"),
        Err(e) => return store_error_response(e),
    };
    let posting = match store.get_posting(req.posting_id).await {
        Ok(Some(p)) => p,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "posting not found"),
        Err(e) => return store_error_response(e),
    };
    let assignment = TaskAssignment {
        assignment_id: Uuid::now_v7(),
        posting_id: req.posting_id,
        template_id: req.template_id,
        due_from: req.due_window.from,
        due_to: req.due_window.to,
        status: TaskStatus::Pending,
        evidence_hash: None,
        block_reason: None,
        updated_at: Utc::now(),
    };
    match store.insert_assignment(&assignment).await {
        Ok(()) => {
            // Just-in-time training: when the task template links training
            // content, surface it to the worker immediately (design §6).
            if let Some(content_id) = template.training_content_id.as_deref() {
                record_notification(
                    &store,
                    posting.worker_id,
                    NotificationKind::Training,
                    format!("Training: {}", template.title),
                    format!(
                        "A new task '{}' is due. Complete the just-in-time training first: {}",
                        template.title, content_id
                    ),
                )
                .await;
            }
            (StatusCode::CREATED, Json(assignment)).into_response()
        }
        Err(e) => store_error_response(e),
    }
}

pub async fn ack_task(State(store): State<Store>, Path(id): Path<Uuid>) -> Response {
    let assignment = match store.get_assignment(id).await {
        Ok(Some(a)) => a,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "assignment not found"),
        Err(e) => return store_error_response(e),
    };
    // Idempotent keyed on (assignment, target-status).
    if assignment.status == TaskStatus::Acked {
        return Json(assignment).into_response();
    }
    if assignment.status != TaskStatus::Pending {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "cannot ack assignment in status '{}'",
                assignment.status.as_str()
            ),
        );
    }
    match store
        .set_assignment_status(id, TaskStatus::Acked, None, None, Utc::now())
        .await
    {
        Ok(a) => Json(a).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn done_task(
    State(store): State<Store>,
    Path(id): Path<Uuid>,
    Json(req): Json<DoneTaskRequest>,
) -> Response {
    let assignment = match store.get_assignment(id).await {
        Ok(Some(a)) => a,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "assignment not found"),
        Err(e) => return store_error_response(e),
    };
    if assignment.status == TaskStatus::Done {
        return Json(assignment).into_response();
    }
    if assignment.status != TaskStatus::Acked {
        return problem(
            StatusCode::CONFLICT,
            format!(
                "cannot complete assignment in status '{}'",
                assignment.status.as_str()
            ),
        );
    }
    let critical = store
        .get_template(assignment.template_id)
        .await
        .ok()
        .flatten()
        .map(|t| t.critical)
        .unwrap_or(false);
    if critical {
        // Critical task completion evidence feeds the transparency log.
        match req.evidence_hash.as_deref() {
            Some(h) if is_hex64(h) => {}
            _ => {
                return problem(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "critical task completion requires a 64-hex evidence_hash",
                )
            }
        }
    } else if let Some(h) = req.evidence_hash.as_deref() {
        if !is_hex64(h) {
            return problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                "evidence_hash must be 64-hex when present",
            );
        }
    }
    match store
        .set_assignment_status(
            id,
            TaskStatus::Done,
            req.evidence_hash.as_deref(),
            None,
            Utc::now(),
        )
        .await
    {
        Ok(a) => {
            emit_event(&store, &task_completed_envelope(&a)).await;
            Json(a).into_response()
        }
        Err(e) => store_error_response(e),
    }
}

pub async fn block_task(
    State(store): State<Store>,
    Path(id): Path<Uuid>,
    Json(req): Json<BlockTaskRequest>,
) -> Response {
    if req.reason.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "blocking requires a non-empty reason",
        );
    }
    let assignment = match store.get_assignment(id).await {
        Ok(Some(a)) => a,
        Ok(None) => return problem(StatusCode::NOT_FOUND, "assignment not found"),
        Err(e) => return store_error_response(e),
    };
    if assignment.status == TaskStatus::Blocked {
        return Json(assignment).into_response();
    }
    if assignment.status == TaskStatus::Done {
        return problem(StatusCode::CONFLICT, "cannot block a completed assignment");
    }
    match store
        .set_assignment_status(id, TaskStatus::Blocked, None, Some(&req.reason), Utc::now())
        .await
    {
        Ok(a) => Json(a).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn list_tasks(
    State(store): State<Store>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let posting_id = params.get("posting_id").and_then(|v| v.parse().ok());
    match store
        .list_assignments(posting_id, params.get("status").map(String::as_str))
        .await
    {
        Ok(assignments) => {
            // Enrich with template metadata so the field app can render the
            // task title and deep-link to its just-in-time training content.
            let mut enriched = Vec::with_capacity(assignments.len());
            for a in assignments {
                let template = store.get_template(a.template_id).await.ok().flatten();
                enriched.push(serde_json::json!({
                    "assignment_id": a.assignment_id,
                    "posting_id": a.posting_id,
                    "template_id": a.template_id,
                    "title": template.as_ref().map(|t| t.title.clone()).unwrap_or_default(),
                    "critical": template.as_ref().map(|t| t.critical).unwrap_or(false),
                    "training_content_id": template.as_ref().and_then(|t| t.training_content_id.clone()),
                    "due_from": a.due_from,
                    "due_to": a.due_to,
                    "status": a.status,
                    "evidence_hash": a.evidence_hash,
                    "block_reason": a.block_reason,
                    "updated_at": a.updated_at,
                }));
            }
            Json(serde_json::json!({ "assignments": enriched })).into_response()
        }
        Err(e) => store_error_response(e),
    }
}

// ---------------------------------------------------------------------------
// Performance
// ---------------------------------------------------------------------------

pub async fn add_record(State(store): State<Store>, Json(req): Json<AddRecordRequest>) -> Response {
    info!(worker_id = %req.worker_id, category = req.category.as_str(), "add_performance_record");
    if store
        .get_worker(req.worker_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return problem(StatusCode::NOT_FOUND, "worker not found");
    }
    let record = PerformanceRecord {
        record_id: Uuid::now_v7(),
        worker_id: req.worker_id,
        category: req.category,
        score: req.score,
        detail: req.detail,
        recorded_by: req.recorded_by,
        recorded_at: Utc::now(),
    };
    match store.insert_record(&record).await {
        Ok(()) => {
            emit_event(&store, &performance_recorded_envelope(&record)).await;
            (StatusCode::CREATED, Json(record)).into_response()
        }
        Err(e) => store_error_response(e),
    }
}

pub async fn get_performance(
    State(store): State<Store>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(worker_id) = params.get("worker_id").and_then(|v| v.parse().ok()) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "worker_id query parameter is required",
        );
    };
    let records = match store.list_records(worker_id).await {
        Ok(r) => r,
        Err(e) => return store_error_response(e),
    };
    let scores: Vec<f32> = records.iter().filter_map(|r| r.score).collect();
    let avg_score = if scores.is_empty() {
        None
    } else {
        Some(scores.iter().sum::<f32>() / scores.len() as f32)
    };
    let incident_count = records
        .iter()
        .filter(|r| r.category == PerfCategory::Incident)
        .count() as i64;
    Json(PerformanceResponse {
        summary: PerfSummary {
            avg_score,
            incident_count,
        },
        records,
    })
    .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ReadinessQuery {
    pub station: Option<String>,
    pub county: Option<String>,
}

/// Minimum staffing per polling station (design §1): one PO, one DPO, two
/// clerks. Readiness reports coverage against this floor.
const MIN_STAFFING: [(&str, u64); 3] = [("PO", 1), ("DPO", 1), ("CLERK", 2)];

/// GET /v1/workforce/readiness — operational rollups for dashboards.
///
/// Composes active postings, open equipment claims and task states into
/// per-station rows; `station` filters to one station when supplied.
pub async fn readiness(
    State(store): State<Store>,
    Query(params): Query<ReadinessQuery>,
) -> Response {
    let mut postings = match store
        .list_postings(None, params.station.as_deref(), Some(true))
        .await
    {
        Ok(p) => p,
        Err(e) => return store_error_response(e),
    };

    // Map worker -> home county for county-scoped readiness.
    let workers = match store.list_workers(None, None).await {
        Ok(w) => w,
        Err(e) => return store_error_response(e),
    };
    let worker_county: std::collections::HashMap<Uuid, String> = workers
        .iter()
        .map(|w| (w.worker_id, w.home_county.clone()))
        .collect();
    if let Some(county) = params.county.as_deref() {
        postings.retain(|p| {
            worker_county
                .get(&p.worker_id)
                .map(|c| c == county)
                .unwrap_or(false)
        });
    }

    let open_claims = match store.list_claims(true).await {
        Ok(c) => c,
        Err(e) => return store_error_response(e),
    };
    let assignments = match store.list_assignments(None, None).await {
        Ok(a) => a,
        Err(e) => return store_error_response(e),
    };
    let now = Utc::now();
    use crate::domain::TaskStatus;
    let mut stations = std::collections::BTreeMap::new();
    for p in &postings {
        let e = stations.entry(p.station_code.clone()).or_insert_with(|| {
            serde_json::json!({
                "station_code": p.station_code,
                "active_postings": 0,
                "staffing": {},
                "staffing_shortfalls": [],
                "coverage_pct": 0.0,
                "open_kit_claims": 0,
                "tasks_total": 0,
                "tasks_done": 0,
                "tasks_blocked": 0,
                "tasks_overdue": 0,
            })
        });
        if let Some(n) = e.get("active_postings").and_then(|v| v.as_i64()) {
            e["active_postings"] = serde_json::json!(n + 1);
        }
        let role = p.role_code.to_uppercase();
        if let Some(obj) = e.get_mut("staffing").and_then(|v| v.as_object_mut()) {
            let cur = obj.get(&role).and_then(|v| v.as_u64()).unwrap_or(0);
            obj.insert(role, serde_json::json!(cur + 1));
        }
    }
    for c in &open_claims {
        if let Some(p) = postings.iter().find(|p| p.posting_id == c.posting_id) {
            if let Some(e) = stations.get_mut(&p.station_code) {
                if let Some(n) = e.get("open_kit_claims").and_then(|v| v.as_i64()) {
                    e["open_kit_claims"] = serde_json::json!(n + 1);
                }
            }
        }
    }
    for a in &assignments {
        let station = postings
            .iter()
            .find(|p| p.posting_id == a.posting_id)
            .map(|p| p.station_code.clone());
        let Some(station) = station else { continue };
        if let Some(e) = stations.get_mut(&station) {
            for key in ["tasks_total"] {
                if let Some(n) = e.get(key).and_then(|v| v.as_i64()) {
                    e[key] = serde_json::json!(n + 1);
                }
            }
            match a.status {
                TaskStatus::Done => bump(e, "tasks_done"),
                TaskStatus::Blocked => bump(e, "tasks_blocked"),
                TaskStatus::Pending | TaskStatus::Acked if a.due_to < now => {
                    bump(e, "tasks_overdue")
                }
                _ => {}
            }
        }
    }

    // Staffing coverage against the per-station floor.
    for e in stations.values_mut() {
        let staffing = e
            .get("staffing")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let counts = staffing.as_object().cloned().unwrap_or_default();
        let mut shortfalls: Vec<String> = Vec::new();
        let mut required_units = 0u64;
        let mut filled_units = 0u64;
        for (role, min) in MIN_STAFFING {
            required_units += min;
            let have = counts.get(role).and_then(|v| v.as_u64()).unwrap_or(0);
            filled_units += have.min(min);
            if have < min {
                shortfalls.push(format!("{role}:{have}/{min}"));
            }
        }
        let coverage_pct = if required_units == 0 {
            100.0
        } else {
            (filled_units as f64 / required_units as f64) * 100.0
        };
        e["staffing"] = serde_json::json!(counts);
        e["staffing_shortfalls"] = serde_json::json!(shortfalls);
        e["coverage_pct"] = serde_json::json!(coverage_pct);
    }

    // County-level rollup for Returning Officers.
    let mut county_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    for p in &postings {
        if let Some(c) = worker_county.get(&p.worker_id) {
            *county_counts.entry(c.clone()).or_insert(0) += 1;
        }
    }
    let counties: Vec<serde_json::Value> = county_counts
        .into_iter()
        .map(|(county, active_postings)| {
            serde_json::json!({ "county": county, "active_postings": active_postings })
        })
        .collect();

    Json(serde_json::json!({
        "stations": stations.values().collect::<Vec<_>>(),
        "counties": counties,
        "totals": {
            "active_postings": postings.len(),
            "open_kit_claims": open_claims.len(),
            "assignments": assignments.len(),
        }
    }))
    .into_response()
}

fn bump(obj: &mut serde_json::Value, key: &str) {
    if let Some(v) = obj.get(key).and_then(|v| v.as_i64()) {
        obj[key] = serde_json::json!(v + 1);
    }
}

// ---------------------------------------------------------------------------
// Audit trail
// ---------------------------------------------------------------------------

/// GET /v1/workforce/audit — staff audit trail (recruitment stage changes and
/// contact-point changes), newest first.
pub async fn audit_trail(State(store): State<Store>) -> Response {
    let stage_changes = match store.list_stage_changes().await {
        Ok(s) => s,
        Err(e) => return store_error_response(e),
    };
    let contact_changes = match store.list_all_contact_log().await {
        Ok(c) => c,
        Err(e) => return store_error_response(e),
    };
    Json(serde_json::json!({
        "stage_changes": stage_changes
            .iter()
            .map(|(case_id, ch)| serde_json::json!({
                "case_id": case_id,
                "from": ch.from,
                "to": ch.to,
                "actor": ch.actor,
                "at": ch.at,
                "reason": ch.reason,
            }))
            .collect::<Vec<_>>(),
        "contact_changes": contact_changes,
    }))
    .into_response()
}

// ---------------------------------------------------------------------------
// Worker profile update
// ---------------------------------------------------------------------------

pub async fn update_worker(
    State(store): State<Store>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkerRequest>,
) -> Response {
    if store.get_worker(id).await.ok().flatten().is_none() {
        return problem(StatusCode::NOT_FOUND, "worker not found");
    }
    if let Some(nok) = &req.next_of_kin {
        if let Some(resp) = hash_problem(&nok.phone_hash, "next_of_kin.phone_hash") {
            return resp;
        }
    }
    match store
        .update_worker(
            id,
            req.home_county,
            req.role_eligibility,
            req.payroll_contract_ref,
            req.next_of_kin.as_ref().map(|n| n.name.clone()),
            req.next_of_kin.as_ref().map(|n| n.phone_hash.clone()),
        )
        .await
    {
        Ok(worker) => Json(worker).into_response(),
        Err(e) => store_error_response(e),
    }
}

// ---------------------------------------------------------------------------
// Interviews
// ---------------------------------------------------------------------------

pub async fn create_interview(
    State(store): State<Store>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateInterviewRequest>,
) -> Response {
    if req.interviewer.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "interviewer is required");
    }
    if store.get_case(id).await.ok().flatten().is_none() {
        return problem(StatusCode::NOT_FOUND, "recruitment case not found");
    }
    let record = InterviewRecord {
        interview_id: Uuid::now_v7(),
        case_id: id,
        interviewer: req.interviewer,
        scheduled_at: req.scheduled_at,
        score: req.score,
        notes: req.notes,
        outcome: req.outcome,
        recorded_at: Utc::now(),
    };
    match store.insert_interview(&record).await {
        Ok(()) => (StatusCode::CREATED, Json(record)).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn list_interviews(State(store): State<Store>, Path(id): Path<Uuid>) -> Response {
    match store.list_interviews(id).await {
        Ok(records) => Json(serde_json::json!({ "interviews": records })).into_response(),
        Err(e) => store_error_response(e),
    }
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

pub async fn send_notification(
    State(store): State<Store>,
    Json(req): Json<SendNotificationRequest>,
) -> Response {
    if req.subject.trim().is_empty() || req.body.trim().is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "subject and body are required",
        );
    }
    if store
        .get_worker(req.worker_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return problem(StatusCode::NOT_FOUND, "worker not found");
    }
    // Resolve the delivery target from the worker's preferred contact point
    // when no explicit channel/delivery_ref was supplied.
    let (channel, delivery_ref) = if let Some(ch) = req.channel {
        (Some(ch), req.delivery_ref)
    } else {
        match store.preferred_contact(req.worker_id).await {
            Ok(Some(c)) => (Some(c.channel), Some(c.value_hash_hex)),
            _ => (None, req.delivery_ref),
        }
    };
    let notification = Notification {
        notification_id: Uuid::now_v7(),
        worker_id: req.worker_id,
        kind: req.kind,
        channel,
        subject: req.subject,
        body: req.body,
        delivery_ref,
        created_at: Utc::now(),
    };
    match store.insert_notification(&notification).await {
        Ok(()) => (StatusCode::CREATED, Json(notification)).into_response(),
        Err(e) => store_error_response(e),
    }
}

pub async fn list_notifications(
    State(store): State<Store>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let Some(worker_id) = params.get("worker_id").and_then(|v| v.parse().ok()) else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "worker_id query parameter is required",
        );
    };
    match store.list_notifications(worker_id).await {
        Ok(notifications) => {
            Json(serde_json::json!({ "notifications": notifications })).into_response()
        }
        Err(e) => store_error_response(e),
    }
}
