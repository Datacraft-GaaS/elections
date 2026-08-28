//! POST /api/v1/bvi/kits/{serial}/revoke and GET /api/v1/bvi/kits/health.

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::{
    error::ApiError,
    idem,
    models::{KitsHealthQuery, RevokeKitRequest, REVOKE_REASONS},
    routes::store_error,
    state::AppState,
    store::BviStore,
};

const REVOKE_INSTANCE: &str = "/api/v1/bvi/kits/{serial}/revoke";
const HEALTH_INSTANCE: &str = "/api/v1/bvi/kits/health";

/// POST /api/v1/bvi/kits/{serial}/revoke (revokeKit).
///
/// Blacklists the kit serial; idempotent per Idempotency-Key.
pub async fn revoke_kit<S: BviStore>(
    State(state): State<AppState<S>>,
    Path(serial): Path<String>,
    headers: HeaderMap,
    body: Result<Json<RevokeKitRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, ApiError> {
    let key = idem::require_key(&headers, REVOKE_INSTANCE)?;
    if let Some(replayed) = idem::replay(&state, "revoke_kit", key).await? {
        return Ok(replayed);
    }

    let Json(req) = body.map_err(|rej| ApiError::bad_request(REVOKE_INSTANCE, rej.body_text()))?;
    if !REVOKE_REASONS.contains(&req.reason.as_str()) {
        return Err(ApiError::bad_request(
            REVOKE_INSTANCE,
            format!("reason must be one of {}", REVOKE_REASONS.join(", ")),
        ));
    }
    if req.officer_id.is_empty() {
        return Err(ApiError::bad_request(REVOKE_INSTANCE, "officer_id is required"));
    }

    let revoked = state
        .store
        .revoke_kit(&serial, &req.reason, &req.officer_id)
        .await
        .map_err(|e| store_error(e, REVOKE_INSTANCE))?;

    idem::snapshot(&state, "revoke_kit", key, &revoked).await?;
    Ok((StatusCode::OK, Json(revoked)).into_response())
}

#[derive(Debug, Deserialize)]
pub struct KitsHealthParams {
    pub station_code: Option<String>,
    pub online: Option<bool>,
    pub cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<u32>,
}

/// GET /api/v1/bvi/kits/health (listKitsHealth).
///
/// Kit health telemetry for NOC dashboards; cursor paginated.
pub async fn list_kits_health<S: BviStore>(
    State(state): State<AppState<S>>,
    headers: HeaderMap,
    Query(params): Query<KitsHealthParams>,
) -> Result<Response, ApiError> {
    idem::require_bearer(&headers, HEALTH_INSTANCE)?;

    let q = KitsHealthQuery {
        station_code: params.station_code.filter(|s| !s.is_empty()),
        online: params.online,
        cursor: params.cursor.filter(|s| !s.is_empty()),
        page_size: params.page_size.unwrap_or(50),
    };

    let page = state
        .store
        .kits_health(&q)
        .await
        .map_err(|e| store_error(e, HEALTH_INSTANCE))?;

    let next_cursor = page.next_cursor.clone();
    let mut res = (StatusCode::OK, Json(page)).into_response();
    if let Some(cursor) = &next_cursor {
        res.headers_mut().insert(
            header::LINK,
            format!("/api/v1/bvi/kits/health?cursor={cursor}").parse().unwrap(),
        );
    }
    Ok(res)
}
