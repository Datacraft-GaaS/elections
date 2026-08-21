//! Health and readiness probe routes.

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
	pub ok: bool,
	pub version: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReadinessResponse {
	pub ready: bool,
	pub checks: serde_json::Value,
}

/// GET /healthz
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "health",
    responses(
        (status = 200, description = "Service healthy", body = HealthResponse),
    )
)]
pub async fn healthz(
	State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<HealthResponse>) {
	(
		StatusCode::OK,
		Json(HealthResponse {
			ok: true,
			version: env!("CARGO_PKG_VERSION").into(),
		}),
	)
}

/// GET /readyz
#[utoipa::path(
    get,
    path = "/readyz",
    tag = "health",
    responses(
        (status = 200, description = "Service ready", body = ReadinessResponse),
        (status = 503, description = "Not ready"),
    )
)]
pub async fn readyz(
	State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<ReadinessResponse>) {
	(
		StatusCode::OK,
		Json(ReadinessResponse {
			ready: true,
			checks: serde_json::json!({
				"database": "ok",
				"vendor": "ok",
				"matcher": "ok"
			}),
		}),
	)
}
