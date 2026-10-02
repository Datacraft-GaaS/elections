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

use crate::{
    error::ApiError,
    models::{AppealGrounds, AppealRecord, CaseStatus, IntakeChannel},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

use super::cases::case_status_from_str;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/dereg/appeals", post(file_appeal))
}

/// Wire representation of an IntakeChannel (matches the SCREAMING_SNAKE_CASE
/// serde renaming and the intake_channel CHECK constraint in V052).
fn intake_channel_str(channel: &IntakeChannel) -> &'static str {
    use IntakeChannel::*;
    match channel {
        VerifierApp => "VERIFIER_APP",
        Web => "WEB",
        Ussd => "USSD",
        InPerson => "IN_PERSON",
    }
}

/// Wire representation of an AppealGrounds (matches the SCREAMING_SNAKE_CASE
/// serde renaming and the grounds CHECK constraint in V052).
fn appeal_grounds_str(grounds: &AppealGrounds) -> &'static str {
    use AppealGrounds::*;
    match grounds {
        IdentityError => "IDENTITY_ERROR",
        AliveAndPresent => "ALIVE_AND_PRESENT",
        DataMatchError => "DATA_MATCH_ERROR",
        FamilyDispute => "FAMILY_DISPUTE",
        Other => "OTHER",
    }
}

/// POST /v1/dereg/appeals
///
/// Files an appeal against a deregistration case. The 30+14-day workflow
/// (see crate::workflow) only allows filing while the case is Noticed and
/// before its notice_deadline; the case then moves to APPEAL_FILED with the
/// appeal id attached. Both writes happen in one transaction.
async fn file_appeal(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<AppealRecord>), ApiError> {
    if !body.is_object() {
        return Err(ApiError::BadRequest("appeal body must be a JSON object".into()));
    }

    let mut val = body;
    if val.get("id").is_none() {
        val["id"] = json!(Uuid::now_v7());
    }
    if val.get("filedAt").is_none() {
        val["filedAt"] = json!(Utc::now());
    }
    let appeal: AppealRecord = serde_json::from_value(val)
        .map_err(|e| ApiError::BadRequest(format!("invalid appeal payload: {e}")))?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let case: Option<(String, Option<DateTime<Utc>>)> =
        sqlx::query_as("SELECT status, notice_deadline FROM dereg_cases WHERE id = $1 FOR UPDATE")
            .bind(appeal.case_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

    let Some((status_s, notice_deadline)) = case else {
        return Err(ApiError::NotFound(format!(
            "case {} not found",
            appeal.case_id
        )));
    };
    if case_status_from_str(&status_s) != Some(CaseStatus::Noticed) {
        return Err(ApiError::Conflict(format!(
            "case status {status_s} does not allow filing an appeal"
        )));
    }
    if notice_deadline.is_some_and(|deadline| Utc::now() > deadline) {
        return Err(ApiError::Conflict("the 30-day appeal window has closed".into()));
    }

    sqlx::query(
        "INSERT INTO dereg_appeals (id, case_id, intake_channel, appellant_id_hash, \
         representative_auth_file_id, grounds, grounds_detail, supporting_doc_ids, \
         language, filed_at, hearing_date) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
    )
    .bind(appeal.id)
    .bind(appeal.case_id)
    .bind(intake_channel_str(&appeal.intake_channel))
    .bind(&appeal.appellant_id_hash)
    .bind(appeal.representative_auth_file_id)
    .bind(appeal_grounds_str(&appeal.grounds))
    .bind(&appeal.grounds_detail)
    .bind(&appeal.supporting_doc_ids)
    .bind(&appeal.language)
    .bind(appeal.filed_at)
    .bind(appeal.hearing_date)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    sqlx::query("UPDATE dereg_cases SET status = 'APPEAL_FILED', appeal_id = $2 WHERE id = $1")
        .bind(appeal.case_id)
        .bind(appeal.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    tracing::info!(appeal_id = %appeal.id, case_id = %appeal.case_id, "appeal filed");
    Ok((StatusCode::CREATED, Json(appeal)))
}
