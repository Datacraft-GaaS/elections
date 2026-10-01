//! Health and readiness probe routes.

use std::sync::Arc;

use axum::{
	extract::State,
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
	pub ok: bool,
	pub version: String,
	/// A real count from the store, so the probe is an observation.
	pub enrolment_jobs_held: usize,
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
	State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<HealthResponse>) {
	// Liveness, not readiness: the request itself proves the process is up and
	// answering, which is the honest thing for a liveness probe to report. The
	// store read is included so the response is an observation rather than a
	// constant, and so a wedged lock shows up here.
	let jobs = state.store.job_count();
	(
		StatusCode::OK,
		Json(HealthResponse {
			ok: true,
			version: env!("CARGO_PKG_VERSION").into(),
			enrolment_jobs_held: jobs,
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
	State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<ReadinessResponse>) {
	// Previously answered `ready: true` with `checks: { database: "ok", vendor: "ok",
	// matcher: "ok" }` without touching the database, a vendor or a matcher. A
	// readiness probe that always passes keeps a broken instance in the
	// load-balancer pool, which is the whole point of having one.
	//
	// The database is genuinely probed, with a short timeout so a black-holed
	// connection cannot hang the probe.
	let db = tokio::time::timeout(
		std::time::Duration::from_millis(500),
		sqlx::query("SELECT 1").execute(&state.db),
	)
	.await;
	let (db_ok, db_detail) = match &db {
		Ok(Ok(_)) => ("ok", Value::Null),
		Ok(Err(e)) => ("unavailable", Value::String(e.to_string())),
		Err(_) => (
			"timeout",
			Value::String("the database did not answer within 500ms".to_string()),
		),
	};

	// The vendor and matcher checks are named as not-performed rather than "ok".
	// A kit vendor is registered by an enrolment, and the matcher is a separate
	// service; neither is reachable from this process, so reporting "ok" would be
	// the same fabrication as before.
	let vendors = state.store.vendors();
	let vendor_status = if vendors.is_empty() {
		("no-vendors-enrolled", Value::Null)
	} else {
		("ok", Value::from(vendors.len()))
	};
	let checks = serde_json::json!({
		"database": db_ok,
		"database_detail": db_detail,
		"vendor": vendor_status.0,
		"vendors_enrolled": vendor_status.1,
		"matcher": "not-performed",
	});
	// Only the database probe is a real check, so only it can gate readiness.
	let ready = db_ok == "ok";
	(
		if ready { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE },
		Json(ReadinessResponse { ready, checks }),
	)
}
