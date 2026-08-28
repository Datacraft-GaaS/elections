//! POST /api/v1/bvi/sessions and PATCH /api/v1/bvi/sessions/{id}/close.

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    extract::rejection::JsonRejection,
    Json,
};
use chrono::Utc;

use crate::{
    error::ApiError,
    idem,
    models::{CloseSessionRequest, SessionCreate},
    routes::store_error,
    state::AppState,
    store::BviStore,
};

const INSTANCE: &str = "/api/v1/bvi/sessions";

/// POST /api/v1/bvi/sessions (openBviSession).
///
/// One active session per device per election day; a second open returns 409.
/// Idempotency-Key mandatory: duplicates replay the original session at 200.
pub async fn open_bvi_session<S: BviStore>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    body: Result<Json<SessionCreate>, JsonRejection>,
) -> Result<Response, ApiError> {
    let key = idem::require_key(&headers, INSTANCE)?;
    if let Some(replayed) = idem::replay(&state, "open_bvi_session", key).await? {
        return Ok(replayed);
    }

    let Json(create) = body.map_err(|rej| ApiError::bad_request(INSTANCE, rej.body_text()))?;
    if create.station_code.is_empty() || create.device_serial.is_empty() || create.election_id.is_empty() {
        return Err(ApiError::bad_request(
            INSTANCE,
            "station_code, device_serial and election_id must be non-empty",
        ));
    }

    let session = state
        .store
        .open_session(&create)
        .await
        .map_err(|e| store_error(e, INSTANCE))?;

    idem::snapshot(&state, "open_bvi_session", key, &session).await?;
    Ok((
        StatusCode::CREATED,
        [(
            header::LOCATION,
            format!("/api/v1/bvi/sessions/{}", session.session_id),
        )],
        Json(session),
    )
        .into_response())
}

/// PATCH /api/v1/bvi/sessions/{id}/close (closeBviSession).
///
/// Closing an already-closed session returns 200 with the existing record.
pub async fn close_bvi_session<S: BviStore>(
    State(state): State<AppState<S>>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    body: Option<Json<CloseSessionRequest>>,
) -> Result<Response, ApiError> {
    let key = idem::require_key(&headers, INSTANCE)?;

    let existing = state
        .store
        .get_session(&session_id)
        .await
        .map_err(|e| store_error(e, INSTANCE))?
        .ok_or_else(|| ApiError::not_found(INSTANCE))?;

    // Replay after the existence check so unknown ids still 404.
    if let Some(replayed) = idem::replay(&state, "close_bvi_session", key).await? {
        return Ok(replayed);
    }

    if existing.status == "active" {
        let closed_at = body.and_then(|Json(b)| b.closed_at).unwrap_or_else(Utc::now);
        let session = state
            .store
            .close_session(&session_id, closed_at)
            .await
            .map_err(|e| store_error(e, INSTANCE))?;
        idem::snapshot(&state, "close_bvi_session", key, &session).await?;
        return Ok((StatusCode::OK, Json(session)).into_response());
    }

    // Already closed or revoked: idempotent close, do not overwrite timestamps.
    Ok((StatusCode::OK, Json(existing)).into_response())
}
