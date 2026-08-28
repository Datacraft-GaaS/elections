//! POST /api/v1/bvi/identifications (recordIdentification).

use axum::{
    extract::{rejection::JsonRejection, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    error::ApiError,
    idem,
    models::{decision_to_outcome, Identification, IdentificationCreate, MODALITIES},
    routes::{intake_guard, store_error},
    state::AppState,
    store::{BviStore, IdentificationNew},
};

const INSTANCE: &str = "/api/v1/bvi/identifications";

/// POST /api/v1/bvi/identifications.
///
/// Records the identification event against the mirror table with a row-level
/// audit entry. The session must exist and be active (404 / 422). A second
/// matched identification for the same voter in the same election is a 409
/// already_identified (EC-OPS-02, enforced by a partial unique index).
pub async fn record_identification<S: BviStore>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    body: Result<Json<IdentificationCreate>, JsonRejection>,
) -> Result<Response, ApiError> {
    let key = idem::require_key(&headers, INSTANCE)?;
    if let Some(replayed) = idem::replay(&state, "record_identification", key).await? {
        return Ok(replayed);
    }

    // HM2 time-bound intake guard. With EEL_POLL_WINDOW unset this is always
    // Decision::Open, so behaviour is identical to an unguarded deployment.
    let mut review_drift: Option<i64> = None;
    match intake_guard::check_window(intake_guard::poll_window(), Utc::now()) {
        intake_guard::Decision::Open => {}
        intake_guard::Decision::Review { drift_secs } => {
            tracing::warn!(
                drift_secs,
                "identification accepted outside poll window; flagged for review"
            );
            review_drift = Some(drift_secs);
        }
        intake_guard::Decision::BeforePoll { opens_in_secs } => {
            return Ok(poll_closed_response(format!("poll opens in {opens_in_secs}s")));
        }
        intake_guard::Decision::AfterPoll { closed_ago_secs } => {
            return Ok(poll_closed_response(format!("poll closed {closed_ago_secs}s ago")));
        }
    }

    let Json(req) = body.map_err(|rej| ApiError::bad_request(INSTANCE, rej.body_text()))?;

    let outcome = decision_to_outcome(&req.decision).ok_or_else(|| {
        ApiError::bad_request(INSTANCE, "decision must be one of match|no_match|fallback_used|ambiguous")
    })?;
    if !(0.0..=1.0).contains(&req.match_score) {
        return Err(ApiError::bad_request(INSTANCE, "match_score must be within [0.0, 1.0]"));
    }
    let modality = req.modality.as_deref().unwrap_or("fingerprint");
    let modality = MODALITIES
        .iter()
        .find(|m| **m == modality)
        .ok_or_else(|| ApiError::bad_request(INSTANCE, "unknown modality"))?;

    let session = state
        .store
        .get_session(&req.session_id)
        .await
        .map_err(|e| store_error(e, INSTANCE))?
        .ok_or_else(|| ApiError::not_found(INSTANCE))?;
    if session.status != "active" {
        return Err(ApiError::unprocessable(
            INSTANCE,
            "Cannot record an identification on a closed session.",
        ));
    }

    let event = IdentificationNew {
        event_id: Uuid::now_v7(),
        session_id: req.session_id.clone(),
        voter_id: req.voter_id.clone(),
        biometric_ref: req.biometric_ref.clone(),
        match_score: req.match_score,
        outcome,
        identified_at: req.identified_at,
        device_serial: req.device_serial.clone(),
        station_code: session.station_code,
        election_id: session.election_id,
        modality,
    };

    let identification: Identification = state
        .store
        .record_identification(&event)
        .await
        .map_err(|e| match e {
            crate::store::StoreError::AlreadyIdentified => {
                ApiError::already_identified(INSTANCE, &req.voter_id, &req.session_id)
            }
            other => store_error(other, INSTANCE),
        })?;

    idem::snapshot(&state, "record_identification", key, &identification).await?;
    let mut response = (StatusCode::CREATED, Json(identification)).into_response();
    if let Some(drift) = review_drift {
        if let Ok(value) = HeaderValue::from_str(&format!("drift={drift}")) {
            response.headers_mut().insert("x-eel-intake-review", value);
        }
    }
    Ok(response)
}

/// 403 problem-style body for submissions outside the poll window.
fn poll_closed_response(detail: String) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "poll closed", "detail": detail })),
    )
        .into_response()
}
