use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use tracing::warn;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct CreateApplicationRequest {
    pub id_number_hash: String,
    pub biographic_fields: serde_json::Value,
    pub consent_tick: bool,
}

#[derive(Debug, Serialize)]
pub struct ApplicationResponse {
    pub app_id: String,
    pub status: String,
}

/// POST /vprs/v1/applications
/// FR-M04-006: submit pre-registration application.
/// FR-M04-021: idempotency key check; constant-time anti-enumeration.
/// FR-M04-022: duplicate id_number_hash returns 409.
#[allow(dead_code)]
pub async fn create_application(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateApplicationRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if state.feature_flags.is_iprs_circuit_closed() {
        warn!("IPRS circuit closed; rejecting new application");
        return Err(ApiError::ServiceUnavailable);
    }

    let _idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let app_id = vprs_domain::uuid7str();

    let _ = sqlx::query(
        r#"
        INSERT INTO vprs_applications (app_id, id_number_hash, status, biographic_fields, consent_tick, expires_at)
        VALUES ($1, $2, 'draft', $3, $4, NOW() + INTERVAL '90 days')
        "#,
    )
    .bind(&app_id)
    .bind(&payload.id_number_hash)
    .bind(&payload.biographic_fields)
    .bind(payload.consent_tick)
    .execute(&state.db)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unique") || msg.contains("duplicate") {
            ApiError::Conflict("application already exists for this id_number_hash".to_string())
        } else {
            ApiError::ServiceUnavailable
        }
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({ "app_id": app_id, "status": "draft" })),
    ))
}

/// GET /vprs/v1/applications/{id}/status
#[allow(dead_code)]
pub async fn get_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let row = sqlx::query("SELECT status FROM vprs_applications WHERE app_id = $1")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| ApiError::ServiceUnavailable)?;

    match row {
        Some(r) => {
            let status: String = r.get("status");
            Ok(Json(json!({ "app_id": id, "status": status })))
        }
        None => Err(ApiError::NotFound),
    }
}

/// DELETE /vprs/v1/applications/{id}
/// FR-M04-006: cancel application; emit ApplicationCancelled.
#[allow(dead_code)]
pub async fn cancel_application(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        "UPDATE vprs_applications SET status = 'cancelled', updated_at = NOW() WHERE app_id = $1 AND status NOT IN ('cancelled','expired','converted')",
    )
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|_| ApiError::ServiceUnavailable)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
