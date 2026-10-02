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

use crate::{error::ApiError, models::DeregistrationCase, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

/// Maximum number of cases returned by the list endpoint.
const LIST_LIMIT: i64 = 100;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/dereg/cases", get(list_cases).post(create_case))
        .route("/v1/dereg/cases/{id}", get(get_case).patch(patch_case))
}

/// Wire representation of a CaseStatus (matches the SCREAMING_SNAKE_CASE
/// serde renaming and the status CHECK constraint in V052).
pub(crate) fn case_status_str(status: &crate::models::CaseStatus) -> &'static str {
    use crate::models::CaseStatus::*;
    match status {
        Noticed => "NOTICED",
        AppealFiled => "APPEAL_FILED",
        UnderReview => "UNDER_REVIEW",
        Adjudication => "ADJUDICATION",
        Overturned => "OVERTURNED",
        Confirmed => "CONFIRMED",
        Deregistered => "DEREGISTERED",
        ExpiredNoAppeal => "EXPIRED_NO_APPEAL",
    }
}

/// Inverse of `case_status_str`.
pub(crate) fn case_status_from_str(s: &str) -> Option<crate::models::CaseStatus> {
    use crate::models::CaseStatus::*;
    match s {
        "NOTICED" => Some(Noticed),
        "APPEAL_FILED" => Some(AppealFiled),
        "UNDER_REVIEW" => Some(UnderReview),
        "ADJUDICATION" => Some(Adjudication),
        "OVERTURNED" => Some(Overturned),
        "CONFIRMED" => Some(Confirmed),
        "DEREGISTERED" => Some(Deregistered),
        "EXPIRED_NO_APPEAL" => Some(ExpiredNoAppeal),
        _ => None,
    }
}

fn adjudicator_decision_str(d: &crate::models::AdjudicatorDecision) -> &'static str {
    use crate::models::AdjudicatorDecision::*;
    match d {
        Overturn => "OVERTURN",
        Confirm => "CONFIRM",
    }
}

fn adjudicator_decision_from_str(s: &str) -> Option<crate::models::AdjudicatorDecision> {
    use crate::models::AdjudicatorDecision::*;
    match s {
        "OVERTURN" => Some(Overturn),
        "CONFIRM" => Some(Confirm),
        _ => None,
    }
}

/// Column order of every SELECT/RETURNING over dereg_cases.
type CaseRow = (
    Uuid,                      // id
    Uuid,                      // voter_id
    Uuid,                      // m35_event_id
    String,                    // status
    Option<DateTime<Utc>>,     // notice_dispatched_at
    Option<DateTime<Utc>>,     // notice_deadline
    Option<Uuid>,              // appeal_id
    Option<Uuid>,              // officer_id
    Option<String>,            // officer_recommendation
    Option<Uuid>,              // adjudicator_id
    Option<String>,            // adjudicator_decision
    Option<DateTime<Utc>>,     // adjudicator_decision_at
    Option<String>,            // adjudicator_notes
    Option<DateTime<Utc>>,     // data_minimisation_at
    Option<Uuid>,              // data_minimisation_officer
    DateTime<Utc>,             // created_at
);

const CASE_COLUMNS: &str = "id, voter_id, m35_event_id, status, notice_dispatched_at, \
     notice_deadline, appeal_id, officer_id, officer_recommendation, adjudicator_id, \
     adjudicator_decision, adjudicator_decision_at, adjudicator_notes, \
     data_minimisation_at, data_minimisation_officer, created_at";

fn case_from_row(row: CaseRow) -> Result<DeregistrationCase, ApiError> {
    let (
        id,
        voter_id,
        m35_event_id,
        status_s,
        notice_dispatched_at,
        notice_deadline,
        appeal_id,
        officer_id,
        officer_recommendation,
        adjudicator_id,
        adjudicator_decision_s,
        adjudicator_decision_at,
        adjudicator_notes,
        data_minimisation_at,
        data_minimisation_officer,
        created_at,
    ) = row;

    let status = case_status_from_str(&status_s)
        .ok_or_else(|| ApiError::Internal(format!("unknown case status {status_s}")))?;
    let adjudicator_decision = match adjudicator_decision_s.as_deref() {
        None => None,
        Some(s) => Some(adjudicator_decision_from_str(s).ok_or_else(|| {
            ApiError::Internal(format!("unknown adjudicator decision {s}"))
        })?),
    };

    Ok(DeregistrationCase {
        id,
        voter_id,
        m35_event_id,
        status,
        notice_dispatched_at,
        notice_deadline,
        appeal_id,
        officer_id,
        officer_recommendation,
        adjudicator_id,
        adjudicator_decision,
        adjudicator_decision_at,
        adjudicator_notes,
        data_minimisation_at,
        data_minimisation_officer,
        created_at,
    })
}

async fn list_cases(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let rows: Vec<CaseRow> = sqlx::query_as(&format!(
        "SELECT {CASE_COLUMNS} FROM dereg_cases ORDER BY created_at DESC LIMIT $1"
    ))
    .bind(LIST_LIMIT)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let cases: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let case = case_from_row(row)?;
            serde_json::to_value(case).map_err(|e| ApiError::Internal(e.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(json!({ "cases": cases })))
}

async fn create_case(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<DeregistrationCase>), ApiError> {
    if !body.is_object() {
        return Err(ApiError::BadRequest("case body must be a JSON object".into()));
    }

    let mut val = body;
    if val.get("id").is_none() {
        val["id"] = json!(Uuid::now_v7());
    }
    if val.get("status").is_none() {
        val["status"] = json!("NOTICED");
    }
    if val.get("createdAt").is_none() {
        val["createdAt"] = json!(Utc::now());
    }
    let case: DeregistrationCase = serde_json::from_value(val)
        .map_err(|e| ApiError::BadRequest(format!("invalid case payload: {e}")))?;

    // m35_event_id must reference a known M35 register event (V050 table).
    let known: Option<(Uuid,)> =
        sqlx::query_as("SELECT event_id FROM eel_voter_register_events WHERE event_id = $1")
            .bind(case.m35_event_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    if known.is_none() {
        return Err(ApiError::NotFound(format!(
            "m35_event_id {} does not reference a known voter-register event",
            case.m35_event_id
        )));
    }

    sqlx::query(
        "INSERT INTO dereg_cases (id, voter_id, m35_event_id, status, notice_dispatched_at, \
         notice_deadline, appeal_id, officer_id, officer_recommendation, adjudicator_id, \
         adjudicator_decision, adjudicator_decision_at, adjudicator_notes, \
         data_minimisation_at, data_minimisation_officer, created_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)",
    )
    .bind(case.id)
    .bind(case.voter_id)
    .bind(case.m35_event_id)
    .bind(case_status_str(&case.status))
    .bind(case.notice_dispatched_at)
    .bind(case.notice_deadline)
    .bind(case.appeal_id)
    .bind(case.officer_id)
    .bind(&case.officer_recommendation)
    .bind(case.adjudicator_id)
    .bind(case.adjudicator_decision.as_ref().map(adjudicator_decision_str))
    .bind(case.adjudicator_decision_at)
    .bind(&case.adjudicator_notes)
    .bind(case.data_minimisation_at)
    .bind(case.data_minimisation_officer)
    .bind(case.created_at)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    tracing::info!(case_id = %case.id, voter_id = %case.voter_id, "deregistration case created");
    Ok((StatusCode::CREATED, Json(case)))
}

async fn get_case(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeregistrationCase>, ApiError> {
    let row: Option<CaseRow> =
        sqlx::query_as(&format!("SELECT {CASE_COLUMNS} FROM dereg_cases WHERE id = $1"))
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

    match row {
        Some(row) => Ok(Json(case_from_row(row)?)),
        None => Err(ApiError::NotFound(format!("case {id} not found"))),
    }
}

async fn patch_case(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<DeregistrationCase>, ApiError> {
    let status_val = body.get("status").ok_or_else(|| {
        ApiError::BadRequest("only status updates are supported: body must be {\"status\": \"...\"}".into())
    })?;
    let status_str = status_val
        .as_str()
        .ok_or_else(|| ApiError::BadRequest("status must be a string".into()))?;
    let status = case_status_from_str(status_str)
        .ok_or_else(|| ApiError::BadRequest(format!("unknown case status {status_str}")))?;

    let row: Option<CaseRow> = sqlx::query_as(&format!(
        "UPDATE dereg_cases SET status = $2 WHERE id = $1 RETURNING {CASE_COLUMNS}"
    ))
    .bind(id)
    .bind(case_status_str(&status))
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    match row {
        Some(row) => Ok(Json(case_from_row(row)?)),
        None => Err(ApiError::NotFound(format!("case {id} not found"))),
    }
}
