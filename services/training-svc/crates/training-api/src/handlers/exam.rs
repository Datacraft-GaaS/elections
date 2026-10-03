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

//! HTTP handlers for the online competency exam engine (§3.2).

use crate::exam::{
    compute_receipt_hash, sample_questions, score_session, ExamBlueprint, ExamResult, ExamSession,
    Question, RoleLevel, SessionStatus,
};
use crate::store::ExamStore;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Shared handler state. `ExamStore` is behind an `Arc<RwLock<..>>` so a Pg
/// implementation can later swap in without changing the route surface.
#[derive(Clone)]
pub struct ExamApiState {
    pub store: Arc<RwLock<ExamStore>>,
}

impl ExamApiState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(ExamStore::new())),
        }
    }
}

impl Default for ExamApiState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn router(state: ExamApiState) -> Router {
    Router::new()
        .route("/v1/exams/blueprints", get(get_blueprints))
        .route(
            "/v1/exams/questions",
            post(post_question).get(get_questions),
        )
        .route("/v1/exams/sessions", post(post_session))
        .route("/v1/exams/sessions/{id}/answers", put(put_answers))
        .route("/v1/exams/sessions/{id}/submit", post(submit_session))
        .route("/v1/exams/sessions/{id}/void", post(void_session))
        .route("/v1/exams/results/{session_id}", get(get_result))
        .route("/v1/exams/paper-batch", post(paper_batch))
        .with_state(state)
}

// ---- requests / responses ----

#[derive(Debug, Deserialize)]
pub struct BlueprintFilter {
    pub role_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QuestionFilter {
    pub role_code: Option<String>,
    pub section: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionReq {
    pub application_id: String,
    pub role_code: RoleLevel,
    pub seed_hex: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PutAnswersReq {
    /// question_id -> selected option indexes
    pub answers: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Deserialize)]
pub struct VoidReq {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct PaperBatchReq {
    pub blueprint_id: String,
    pub entries: Vec<PaperBatchEntry>,
}

#[derive(Debug, Deserialize)]
pub struct PaperBatchEntry {
    pub application_id: String,
    pub total_pct: f32,
}

fn error(status: StatusCode, code: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": code })))
}

// ---- blueprints & questions ----

pub async fn get_blueprints(
    State(s): State<ExamApiState>,
    Query(f): Query<BlueprintFilter>,
) -> impl IntoResponse {
    let now = Utc::now();
    let store = s.store.read().await;
    let mut blueprints: Vec<&ExamBlueprint> = store
        .blueprints
        .values()
        .filter(|b| b.is_active_at(now))
        .collect();
    if let Some(role) = &f.role_code {
        if let Ok(level) =
            serde_json::from_value::<RoleLevel>(serde_json::Value::String(role.clone()))
        {
            blueprints.retain(|b| b.role_code == level);
        }
    }
    blueprints.sort_by(|a, b| a.blueprint_id.cmp(&b.blueprint_id));
    (
        StatusCode::OK,
        Json(serde_json::json!({ "blueprints": blueprints })),
    )
}

pub async fn post_question(
    State(s): State<ExamApiState>,
    Json(req): Json<Question>,
) -> impl IntoResponse {
    info!(question_id=%req.question_id, section=%req.section, "post_question");
    let mut store = s.store.write().await;
    store.questions.insert(req.question_id.clone(), req);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "status": "created" })),
    )
}

pub async fn get_questions(
    State(s): State<ExamApiState>,
    Query(f): Query<QuestionFilter>,
) -> impl IntoResponse {
    let store = s.store.read().await;
    let role = f
        .role_code
        .and_then(|r| serde_json::from_value::<RoleLevel>(serde_json::Value::String(r)).ok());
    let mut questions: Vec<&Question> = store
        .questions
        .values()
        .filter(|q| role.is_none_or(|r| q.role_level == r))
        .filter(|q| f.section.as_ref().is_none_or(|sec| &q.section == sec))
        .filter(|q| f.language.as_ref().is_none_or(|lang| &q.language == lang))
        .collect();
    questions.sort_by(|a, b| a.question_id.cmp(&b.question_id));
    // Bank administration view: correct indexes included for authoring tools.
    (
        StatusCode::OK,
        Json(serde_json::json!({ "questions": questions })),
    )
}

// ---- sessions ----

/// Session issued to a candidate. The response carries the sampled question
/// ids WITH options but WITHOUT correct answers.
#[derive(serde::Serialize)]
struct IssuedQuestion<'a> {
    question_id: &'a str,
    section: &'a str,
    difficulty: u8,
    language: &'a str,
    body: &'a str,
    options: &'a [String],
}

pub async fn post_session(
    State(s): State<ExamApiState>,
    Json(req): Json<CreateSessionReq>,
) -> impl IntoResponse {
    let mut store = s.store.write().await;
    let now = Utc::now();
    let blueprint = match store
        .blueprints
        .values()
        .find(|b| b.role_code == req.role_code && b.is_active_at(now))
    {
        Some(b) => b.clone(),
        None => return error(StatusCode::NOT_FOUND, "no_active_blueprint"),
    };

    // Voided sessions consume an attempt too (see store.attempts_used).
    let attempts_used = store.attempts_used(&req.application_id);
    if attempts_used >= blueprint.attempts_allowed {
        return error(StatusCode::CONFLICT, "attempts_exhausted");
    }

    let bank: Vec<Question> = store.questions.values().cloned().collect();
    let seed_hex = req
        .seed_hex
        .clone()
        .unwrap_or_else(|| hex::encode(Uuid::new_v4().as_bytes()));
    let question_ids = sample_questions(&bank, &blueprint, &seed_hex);

    let started_at = Utc::now();
    let session = ExamSession {
        session_id: Uuid::new_v4().to_string(),
        application_id: req.application_id,
        blueprint_id: blueprint.blueprint_id.clone(),
        attempt: attempts_used + 1,
        seed_hex,
        // issued -> in_progress immediately: server-side timing starts here.
        status: SessionStatus::InProgress,
        started_at,
        deadline: started_at + Duration::seconds(blueprint.duration_secs),
        question_ids: question_ids.clone(),
        receipt_hash_hex: None,
    };
    info!(session_id=%session.session_id, application_id=%session.application_id, attempt=session.attempt, "exam session issued");
    store
        .sessions
        .insert(session.session_id.clone(), session.clone());

    let issued: Vec<IssuedQuestion> = question_ids
        .iter()
        .filter_map(|id| store.questions.get(id))
        .map(|q| IssuedQuestion {
            question_id: &q.question_id,
            section: &q.section,
            difficulty: q.difficulty,
            language: &q.language,
            body: &q.body,
            options: &q.options,
        })
        .collect();

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "session": {
                "session_id": session.session_id,
                "application_id": session.application_id,
                "blueprint_id": session.blueprint_id,
                "attempt": session.attempt,
                "seed_hex": session.seed_hex,
                "status": session.status,
                "started_at": session.started_at,
                "deadline": session.deadline,
            },
            "questions": issued,
        })),
    )
}

/// Idempotent answer upsert keyed by (session, question): re-PUT of the same
/// payload is a no-op; re-PUT with a DIFFERENT answer after submit is 409;
/// before submit it replaces the stored answer.
pub async fn put_answers(
    State(s): State<ExamApiState>,
    Path(id): Path<String>,
    Json(req): Json<PutAnswersReq>,
) -> impl IntoResponse {
    let mut store = s.store.write().await;
    let Some(session) = store.sessions.get(&id).cloned() else {
        return error(StatusCode::NOT_FOUND, "session_not_found");
    };
    match session.status {
        SessionStatus::Voided => return error(StatusCode::CONFLICT, "session_voided"),
        SessionStatus::Submitted | SessionStatus::Scored => {
            for (qid, ans) in &req.answers {
                if let Some(existing) = store.answers.get(&(id.clone(), qid.clone())) {
                    if existing != ans {
                        return error(StatusCode::CONFLICT, "already_submitted");
                    }
                } else {
                    return error(StatusCode::CONFLICT, "already_submitted");
                }
            }
            return (
                StatusCode::OK,
                Json(serde_json::json!({ "status": "idempotent" })),
            );
        }
        _ => {}
    }
    for (qid, ans) in req.answers {
        store.answers.insert((id.clone(), qid), ans);
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "accepted" })),
    )
}

/// Submit and auto-score. Deadline is enforced at submit time; late submits
/// get 422 with reason "deadline_elapsed". Re-submitting an already scored
/// session is idempotent and returns the stored result.
pub async fn submit_session(
    State(s): State<ExamApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut store = s.store.write().await;
    let Some(session) = store.sessions.get(&id).cloned() else {
        return error(StatusCode::NOT_FOUND, "session_not_found");
    };
    match session.status {
        SessionStatus::Scored => {
            let result = store.results.get(&id).cloned();
            return match result {
                Some(r) => (
                    StatusCode::OK,
                    Json(serde_json::to_value(r).unwrap_or_default()),
                ),
                None => error(StatusCode::INTERNAL_SERVER_ERROR, "result_missing"),
            };
        }
        SessionStatus::Submitted => {
            return error(StatusCode::CONFLICT, "submit_in_flight");
        }
        SessionStatus::Voided => return error(StatusCode::CONFLICT, "session_voided"),
        _ => {}
    }
    if Utc::now() > session.deadline {
        return error(StatusCode::UNPROCESSABLE_ENTITY, "deadline_elapsed");
    }
    let Some(blueprint) = store.blueprints.get(&session.blueprint_id).cloned() else {
        return error(StatusCode::INTERNAL_SERVER_ERROR, "blueprint_missing");
    };
    let bank: Vec<Question> = store.questions.values().cloned().collect();
    let answers: HashMap<String, Vec<usize>> = store
        .answers
        .iter()
        .filter(|((sid, _), _)| sid == &id)
        .map(|((_, qid), ans)| (qid.clone(), ans.clone()))
        .collect();

    let mut session = session;
    session.status = SessionStatus::Submitted;
    let mut result = score_session(&session, &blueprint, &bank, &answers);
    session.status = SessionStatus::Scored;
    session.receipt_hash_hex = Some(result.signed_receipt_hash_hex.clone());
    result.channel = "digital".to_string();

    info!(session_id=%id, total_pct=result.total_pct, passed=result.passed, "exam submitted");
    store.sessions.insert(id.clone(), session);
    store.results.insert(id.clone(), result.clone());
    (
        StatusCode::OK,
        Json(serde_json::to_value(result).unwrap_or_default()),
    )
}

/// Board-override void. NOTE: voiding CONSUMES the attempt — attempts are
/// counted across all session statuses (see `ExamStore::attempts_used`) so a
/// candidate cannot reset their budget by voiding mid-exam.
pub async fn void_session(
    State(s): State<ExamApiState>,
    Path(id): Path<String>,
    Json(req): Json<VoidReq>,
) -> impl IntoResponse {
    info!(session_id=%id, reason=%req.reason, "void_session");
    let mut store = s.store.write().await;
    match store.sessions.get_mut(&id) {
        None => error(StatusCode::NOT_FOUND, "session_not_found"),
        Some(session) => match session.status {
            SessionStatus::Voided => (
                StatusCode::OK,
                Json(serde_json::json!({ "session_id": id, "status": "voided" })),
            ),
            _ => {
                session.status = SessionStatus::Voided;
                (
                    StatusCode::OK,
                    Json(serde_json::json!({ "session_id": id, "status": "voided" })),
                )
            }
        },
    }
}

pub async fn get_result(
    State(s): State<ExamApiState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let store = s.store.read().await;
    match store.results.get(&session_id) {
        Some(r) => (
            StatusCode::OK,
            Json(serde_json::to_value(r).unwrap_or_default()),
        ),
        None => error(StatusCode::NOT_FOUND, "result_not_found"),
    }
}

/// OMR-style batch import for paper cohorts (§3.2 offline fallback): creates
/// synthetic scored sessions + results marked channel="paper".
pub async fn paper_batch(
    State(s): State<ExamApiState>,
    Json(req): Json<PaperBatchReq>,
) -> impl IntoResponse {
    let mut store = s.store.write().await;
    let Some(blueprint) = store.blueprints.get(&req.blueprint_id).cloned() else {
        return error(StatusCode::NOT_FOUND, "blueprint_not_found");
    };
    let now = Utc::now();
    let mut created = Vec::with_capacity(req.entries.len());
    for entry in &req.entries {
        let attempts_used = store.attempts_used(&entry.application_id);
        let session_id = Uuid::new_v4().to_string();
        let session = ExamSession {
            session_id: session_id.clone(),
            application_id: entry.application_id.clone(),
            blueprint_id: blueprint.blueprint_id.clone(),
            attempt: attempts_used + 1,
            seed_hex: String::new(),
            status: SessionStatus::Scored,
            started_at: now,
            deadline: now,
            question_ids: Vec::new(),
            receipt_hash_hex: None,
        };
        store.sessions.insert(session_id.clone(), session);

        let mut result = ExamResult {
            session_id: session_id.clone(),
            application_id: entry.application_id.clone(),
            blueprint_id: blueprint.blueprint_id.clone(),
            attempt: attempts_used + 1,
            section_scores: Vec::new(),
            total_pct: entry.total_pct,
            passed: entry.total_pct >= blueprint.pass_mark_pct,
            signed_receipt_hash_hex: String::new(),
            channel: String::new(),
        };
        result.channel = "paper".to_string();
        result.signed_receipt_hash_hex = compute_receipt_hash(&result);
        store.results.insert(session_id.clone(), result.clone());
        created.push(result);
    }
    info!(count = created.len(), blueprint_id=%blueprint.blueprint_id, "paper batch imported");
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "results": created })),
    )
}
