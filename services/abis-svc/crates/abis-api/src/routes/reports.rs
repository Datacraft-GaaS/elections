//! Reports routes.

use std::sync::Arc;

use axum::{
	response::IntoResponse,
	extract::{Query, State},
	http::StatusCode,
	Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct FmrQuery {
	pub slice: Option<String>, // "demographic"
}

#[derive(Debug, Deserialize)]
pub struct DedupQuery {
	pub from: Option<String>,
	pub to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FairnessQuery {
	pub dimension: Option<String>, // "sex" | "age" | "county"
}

// ---------------------------------------------------------------------------
// Response models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FmrFnmrReport {
	pub generated_at: String,
	pub slice: Option<String>,
	pub fmr: f64,
	pub fnmr: f64,
	pub threshold: f32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DedupSummary {
	pub from: Option<String>,
	pub to: Option<String>,
	pub total_comparisons: u64,
	pub duplicate_pairs: u64,
	pub duplicate_rate_pct: f32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FairnessReport {
	pub dimension: Option<String>,
	pub generated_at: String,
	pub groups: Vec<FairnessGroup>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FairnessGroup {
	pub label: String,
	pub fmr: f64,
	pub fnmr: f64,
	pub sample_size: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ThroughputReport {
	pub generated_at: String,
	pub enrolments_per_hour: u64,
	pub p50_ms: u32,
	pub p99_ms: u32,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /v1/reports/fmr-fnmr
#[utoipa::path(
    get,
    path = "/v1/reports/fmr-fnmr",
    tag = "reports",
    params(("slice" = Option<String>, Query, description = "Demographic slice")),
    responses(
        (status = 200, description = "FMR/FNMR report", body = FmrFnmrReport),
    )
)]
pub async fn get_fmr_fnmr(
	State(_state): State<Arc<AppState>>,
	Query(_q): Query<FmrQuery>,
) -> ApiError {
	// Previously returned `fmr: 0.001, fnmr: 0.005` as constants, for any slice.
	// False match and false non-match rates are what determine whether a biometric
	// system is fit for electoral use, and they are measured against a test corpus
	// of known genuine and known impostor presentations. There is no such corpus
	// in this workspace and the matcher is not reachable from this process.
	//
	// A 200 with plausible-looking rates is the worst possible answer here: a
	// reader has no way to tell 0.001 from a measured value, and a dashboard will
	// render it. This uses the same explicit refusal as `get_reports` above.
	ApiError::NotImplemented(
		"FMR and FNMR are measured against a test corpus of known genuine and known \
		 impostor presentations; no such corpus is in this workspace and the matcher is \
		 not reachable from this process. It previously reported fmr 0.001 and fnmr 0.005 \
		 as constants for any slice, which is indistinguishable from a measurement"
			.to_string(),
	)
}

/// GET /v1/reports/dedup-summary
#[utoipa::path(
    get,
    path = "/v1/reports/dedup-summary",
    tag = "reports",
    params(
        ("from" = Option<String>, Query, description = "Start datetime (ISO 8601)"),
        ("to" = Option<String>, Query, description = "End datetime (ISO 8601)"),
    ),
    responses(
        (status = 200, description = "Dedup summary", body = DedupSummary),
    )
)]
pub async fn get_dedup_summary(
	State(state): State<Arc<AppState>>,
	Query(q): Query<DedupQuery>,
) -> axum::response::Response {
	// Previously returned zeros for any date range, which reads as "no duplicates
	// were found" rather than "nothing was compared". Deduplication runs in the
	// batch pipeline, which this process does not host.
	let jobs = state.store.jobs_in_window(24 * 30);
	let captured = jobs
		.iter()
		.filter(|j| j.status == crate::store::JobStatus::Captured)
		.count() as u64;
	Json(serde_json::json!({
		"from": q.from,
		"to": q.to,
		"total_comparisons": Value::Null,
		"duplicate_pairs": Value::Null,
		"duplicate_rate_pct": Value::Null,
		"enrolment_jobs_captured": captured,
		"measured": false,
		"blocking_reason": "deduplication runs in the batch dedup pipeline, which this \
							process does not host; captured_jobs is a count of this \
							service's own records and is not a deduplication result",
	}))
	.into_response()
}

/// GET /v1/reports/fairness
#[utoipa::path(
    get,
    path = "/v1/reports/fairness",
    tag = "reports",
    params(("dimension" = Option<String>, Query, description = "sex|age|county")),
    responses(
        (status = 200, description = "Fairness report", body = FairnessReport),
    )
)]
pub async fn get_fairness(
	State(_state): State<Arc<AppState>>,
	Query(_q): Query<FairnessQuery>,
) -> ApiError {
	// Previously returned empty groups, which reads as "no disparity was found".
	ApiError::NotImplemented(
		"a fairness report needs per-demographic FMR and FNMR, which require both the \
		 captured demographics and the matcher; neither is reachable from this process. \
		 It previously returned an empty group list for any dimension, which reads as \
		 no disparity"
			.to_string(),
	)
}

/// GET /v1/reports/enrolment-throughput
#[utoipa::path(
    get,
    path = "/v1/reports/enrolment-throughput",
    tag = "reports",
    responses(
        (status = 200, description = "Enrolment throughput report", body = ThroughputReport),
    )
)]
pub async fn get_enrolment_throughput(
	State(state): State<Arc<AppState>>,
) -> axum::response::Response {
	// Previously returned zeroed percentiles, which reads as a measured latency
	// distribution. A rate of 0 enrolments/hour over an empty window is a true
	// statement about this service; it is not a statement about the kits.
	let jobs = state.store.jobs_in_window(1);
	let captured = jobs
		.iter()
		.filter(|j| j.status == crate::store::JobStatus::Captured)
		.count() as u64;
	Json(serde_json::json!({
		"generated_at": chrono::Utc::now().to_rfc3339(),
		"window_hours": 1,
		"enrolments_per_hour": captured,
		"jobs_accepted_in_window": jobs.len() as u64,
		"p50_ms": Value::Null,
		"p99_ms": Value::Null,
		"measured": false,
		"blocking_reason": "latency is measured at the kit and the matcher; neither is \
							reachable from this process, so the percentiles are null rather \
							than zero. enrolments_per_hour counts this service's own records",
	}))
	.into_response()
}

/// GET /v1/reports (legacy summary endpoint, kept for compat)
pub async fn get_reports(State(_state): State<Arc<AppState>>) -> ApiError {
	// Previously returned 200 with gallery_size: 0, match_throughput_per_hour:
	// Some(0) and duplicate_rate_pct: Some(0.0). Those read as measured
	// operational telemetry, so a dashboard would show a healthy zeroed system
	// rather than an absent one.
	ApiError::NotImplemented(
		"the abis metrics summary is declared by the contract but has no \
		 telemetry source behind it; it previously reported zeroed \
		 throughput and duplicate rate as if measured"
			.to_string(),
	)
}
