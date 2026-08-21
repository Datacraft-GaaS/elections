use axum::{
	extract::{Path, Query, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Json, Response},
	routing::{get, post},
	Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::repository;
use crate::scheduler::next_fire;

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ApiState {
	pub pool: PgPool,
}

// ---------------------------------------------------------------------------
// Request / response models
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
	pub slug:        String,
	pub window_from: chrono::DateTime<chrono::Utc>,
	pub window_to:   chrono::DateTime<chrono::Utc>,
	pub formats:     Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListDefsQuery {
	pub class: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SchedulePreviewResponse {
	pub cron:      String,
	pub next_fire: Option<String>,
	pub error:     Option<String>,
}

// ---------------------------------------------------------------------------
// Error helper
// ---------------------------------------------------------------------------

fn api_err(status: StatusCode, msg: impl ToString) -> Response {
	(status, Json(serde_json::json!({ "error": msg.to_string() }))).into_response()
}

// ---------------------------------------------------------------------------
// Idempotency-Key middleware (thin — just reads the header for now)
// ---------------------------------------------------------------------------

fn idempotency_key(headers: &HeaderMap) -> Option<String> {
	headers
		.get("Idempotency-Key")
		.and_then(|v| v.to_str().ok())
		.map(str::to_owned)
}

// ---------------------------------------------------------------------------
// Handlers — Definitions
// ---------------------------------------------------------------------------

async fn list_definitions(
	State(s): State<ApiState>,
	Query(q): Query<ListDefsQuery>,
) -> Response {
	match repository::fetch_active_definitions(&s.pool, q.class.as_deref()).await {
		Ok(defs) => Json(defs).into_response(),
		Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	}
}

async fn get_definition(
	State(s): State<ApiState>,
	Path(slug): Path<String>,
) -> Response {
	match repository::fetch_definition_by_slug(&s.pool, &slug).await {
		Ok(Some(def)) => Json(def).into_response(),
		Ok(None) => api_err(StatusCode::NOT_FOUND, format!("definition not found: {slug}")),
		Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	}
}

async fn create_definition(
	State(s): State<ApiState>,
	headers: HeaderMap,
	Json(body): Json<crate::domain::ReportDefinition>,
) -> Response {
	let _idempotency = idempotency_key(&headers);
	match repository::insert_definition(&s.pool, &body).await {
		Ok(()) => (StatusCode::CREATED, Json(body)).into_response(),
		Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	}
}

// ---------------------------------------------------------------------------
// Handlers — Runs
// ---------------------------------------------------------------------------

async fn create_run(
	State(s): State<ApiState>,
	headers: HeaderMap,
	Json(body): Json<CreateRunRequest>,
) -> Response {
	let _idempotency = idempotency_key(&headers);
	let def = match repository::fetch_definition_by_slug(&s.pool, &body.slug).await {
		Ok(Some(d)) => d,
		Ok(None) => return api_err(StatusCode::NOT_FOUND, format!("slug not found: {}", body.slug)),
		Err(e) => return api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	};
	match repository::create_run(
		&s.pool,
		def.definition_id,
		&body.slug,
		body.window_from,
		body.window_to,
		"api",
		None,
		None,
	)
	.await
	{
		Ok(run) => (StatusCode::CREATED, Json(run)).into_response(),
		Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	}
}

async fn get_run(
	State(s): State<ApiState>,
	Path(run_id): Path<Uuid>,
) -> Response {
	match repository::fetch_run(&s.pool, run_id).await {
		Ok(Some(run)) => Json(run).into_response(),
		Ok(None) => api_err(StatusCode::NOT_FOUND, format!("run not found: {run_id}")),
		Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	}
}

async fn list_artifacts(
	State(s): State<ApiState>,
	Path(run_id): Path<Uuid>,
) -> Response {
	match repository::fetch_artifacts_by_run(&s.pool, run_id).await {
		Ok(arts) => Json(arts).into_response(),
		Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	}
}

// ---------------------------------------------------------------------------
// Handlers — Artifacts
// ---------------------------------------------------------------------------

async fn get_artifact(
	State(s): State<ApiState>,
	Path(artifact_id): Path<Uuid>,
	headers: HeaderMap,
) -> Response {
	let art = match repository::fetch_artifact(&s.pool, artifact_id).await {
		Ok(Some(a)) => a,
		Ok(None) => return api_err(StatusCode::NOT_FOUND, format!("artifact not found: {artifact_id}")),
		Err(e) => return api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	};

	// Record access
	let ip_prefix = headers
		.get("X-Forwarded-For")
		.and_then(|v| v.to_str().ok())
		.map(|ip| cidr_prefix(ip));
	let ua_hash: Option<Vec<u8>> = headers
		.get("User-Agent")
		.and_then(|v| v.to_str().ok())
		.map(|ua| {
			use sha2::{Digest, Sha256};
			Sha256::digest(ua.as_bytes()).to_vec()
		});

	let access_id = Uuid::now_v7();
	let _ = repository::record_access(
		&s.pool,
		access_id,
		artifact_id,
		None,
		ip_prefix.as_deref(),
		ua_hash.as_deref(),
		None,
	)
	.await;

	Json(art).into_response()
}

async fn verify_artifact(
	State(s): State<ApiState>,
	Path(artifact_id): Path<Uuid>,
) -> Response {
	let art = match repository::fetch_artifact(&s.pool, artifact_id).await {
		Ok(Some(a)) => a,
		Ok(None) => return api_err(StatusCode::NOT_FOUND, format!("artifact not found: {artifact_id}")),
		Err(e) => return api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	};

	let anchor = repository::fetch_tlog_anchor_by_artifact(&s.pool, artifact_id).await.ok().flatten();
	let hsm_verified = crate::signer::verify(
		art.hsm_signature.as_deref().unwrap_or_default(),
		&art.sha256,
		art.hsm_key_role.as_deref().unwrap_or(""),
	)
	.unwrap_or(false);

	Json(serde_json::json!({
		"artifact_id": artifact_id,
		"hsm_signature_verified": hsm_verified,
		"tlog_anchor": anchor,
	}))
	.into_response()
}

// ---------------------------------------------------------------------------
// Schedule dry-run
// ---------------------------------------------------------------------------

async fn schedule_preview(
	State(_s): State<ApiState>,
	Path(slug): Path<String>,
	Json(body): Json<serde_json::Value>,
) -> Response {
	let cron_expr = body
		.get("cron")
		.and_then(|v| v.as_str())
		.unwrap_or("")
		.to_owned();

	let (next, error) = match next_fire(&cron_expr) {
		Ok(Some(t)) => (Some(t.to_rfc3339()), None),
		Ok(None) => (None, None),
		Err(e) => (None, Some(e)),
	};

	Json(SchedulePreviewResponse {
		cron: cron_expr,
		next_fire: next,
		error,
	})
	.into_response()
}

// ---------------------------------------------------------------------------
// Delivery
// ---------------------------------------------------------------------------

async fn get_delivery(
	State(s): State<ApiState>,
	Path(delivery_id): Path<Uuid>,
) -> Response {
	match repository::fetch_delivery(&s.pool, delivery_id).await {
		Ok(Some(d)) => Json(d).into_response(),
		Ok(None) => api_err(StatusCode::NOT_FOUND, format!("delivery not found: {delivery_id}")),
		Err(e) => api_err(StatusCode::INTERNAL_SERVER_ERROR, e),
	}
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(state: ApiState) -> Router {
	Router::new()
		// Definitions
		.route("/v1/reports/definitions", get(list_definitions).post(create_definition))
		.route("/v1/reports/definitions/{slug}", get(get_definition))
		.route("/v1/reports/definitions/{slug}/schedule", post(schedule_preview))
		// Runs
		.route("/v1/reports/runs", post(create_run))
		.route("/v1/reports/runs/{run_id}", get(get_run))
		.route("/v1/reports/runs/{run_id}/artifacts", get(list_artifacts))
		// Artifacts
		.route("/v1/reports/artifacts/{artifact_id}", get(get_artifact))
		.route("/v1/reports/artifacts/{artifact_id}/verify", get(verify_artifact))
		// Deliveries
		.route("/v1/reports/deliveries/{delivery_id}", get(get_delivery))
		.with_state(state)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the /24 (v4) or /48 (v6) prefix of an IP address for privacy.
fn cidr_prefix(ip: &str) -> String {
	let ip = ip.split(',').next().unwrap_or(ip).trim();
	if ip.contains(':') {
		// IPv6: keep first 3 groups
		let parts: Vec<&str> = ip.split(':').collect();
		parts[..parts.len().min(3)].join(":") + "::/48"
	} else {
		// IPv4: keep first 3 octets
		let parts: Vec<&str> = ip.split('.').collect();
		parts[..parts.len().min(3)].join(".") + ".0/24"
	}
}
