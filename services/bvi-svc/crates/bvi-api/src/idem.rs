//! ADR-0011 idempotency plumbing shared by every mutating handler.

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    error::ApiError,
    state::AppState,
    store::BviStore,
};

/// Extract and validate the mandatory Idempotency-Key header (UUID v4/v7).
pub fn require_key(headers: &HeaderMap, instance: &'static str) -> Result<Uuid, ApiError> {
    let raw = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::bad_request(instance, "Idempotency-Key header is required"))?;
    Uuid::parse_str(raw)
        .map_err(|_| ApiError::bad_request(instance, "Idempotency-Key must be a UUID"))
}

/// Return the replayed response for a seen key, if any. Success snapshots are
/// always replayed with status 200 per the contract ("duplicate idempotency
/// key" responses); error outcomes are never snapshotted.
pub async fn replay<S: BviStore>(
    state: &AppState<S>,
    operation: &str,
    key: Uuid,
) -> Result<Option<Response>, ApiError> {
    let hit = state
        .store
        .idem_get(operation, &key.to_string())
        .await
        .map_err(|_| ApiError::internal(operation_instance(operation)))?;
    Ok(hit.map(|(_, body)| {
        match body {
            Some(v) => (StatusCode::OK, Json(v)).into_response(),
            // Snapshot without a body (204-style): duplicate returns 200 empty.
            None => (StatusCode::OK, Body::empty()).into_response(),
        }
    }))
}

/// Persist a success response snapshot for later replay.
pub async fn snapshot<S: BviStore, T: Serialize>(
    state: &AppState<S>,
    operation: &str,
    key: Uuid,
    body: &T,
) -> Result<(), ApiError> {
    let value = serde_json::to_value(body)
        .map_err(|_| ApiError::internal(operation_instance(operation)))?;
    state
        .store
        .idem_put(operation, &key.to_string(), 200, Some(&value))
        .await
        .map_err(|_| ApiError::internal(operation_instance(operation)))?;
    Ok(())
}

fn operation_instance(operation: &str) -> &'static str {
    match operation {
        "open_bvi_session" => "/api/v1/bvi/sessions",
        "close_bvi_session" => "/api/v1/bvi/sessions/{id}/close",
        "record_identification" => "/api/v1/bvi/identifications",
        "submit_telemetry" => "/api/v1/bvi/telemetry",
        "raise_distress" => "/api/v1/bvi/distress",
        "revoke_kit" => "/api/v1/bvi/kits/{serial}/revoke",
        _ => "/api/v1/bvi",
    }
}

/// Tier 2/3 endpoints are JWT bearer at the gateway; the service requires the
/// header to be present and non-empty (Envoy validates the token itself).
pub fn require_bearer(headers: &HeaderMap, instance: &'static str) -> Result<(), ApiError> {
    let ok = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            let mut parts = v.splitn(2, ' ');
            parts.next() == Some("Bearer") && parts.next().map(str::is_empty) == Some(false)
        })
        .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(ApiError::unauthorized(instance))
    }
}
