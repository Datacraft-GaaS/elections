//! Reports routes.

use std::sync::Arc;

use axum::{
	extract::{Query, State},
	http::StatusCode,
	Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::AbisReport;
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
	Query(q): Query<FmrQuery>,
) -> (StatusCode, Json<FmrFnmrReport>) {
	(
		StatusCode::OK,
		Json(FmrFnmrReport {
			generated_at: chrono::Utc::now().to_rfc3339(),
			slice: q.slice,
			fmr: 0.001,
			fnmr: 0.005,
			threshold: 0.8,
		}),
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
	State(_state): State<Arc<AppState>>,
	Query(q): Query<DedupQuery>,
) -> (StatusCode, Json<DedupSummary>) {
	(
		StatusCode::OK,
		Json(DedupSummary {
			from: q.from,
			to: q.to,
			total_comparisons: 0,
			duplicate_pairs: 0,
			duplicate_rate_pct: 0.0,
		}),
	)
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
	Query(q): Query<FairnessQuery>,
) -> (StatusCode, Json<FairnessReport>) {
	(
		StatusCode::OK,
		Json(FairnessReport {
			dimension: q.dimension,
			generated_at: chrono::Utc::now().to_rfc3339(),
			groups: vec![],
		}),
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
	State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<ThroughputReport>) {
	(
		StatusCode::OK,
		Json(ThroughputReport {
			generated_at: chrono::Utc::now().to_rfc3339(),
			enrolments_per_hour: 0,
			p50_ms: 0,
			p99_ms: 0,
		}),
	)
}

/// GET /v1/reports (legacy summary endpoint, kept for compat)
pub async fn get_reports(State(_state): State<Arc<AppState>>) -> (StatusCode, Json<AbisReport>) {
	(
		StatusCode::OK,
		Json(AbisReport {
			generated_at: chrono::Utc::now().to_rfc3339(),
			gallery_size: 0,
			pending_dedup_count: 0,
			open_adjudication_count: 0,
			match_throughput_per_hour: 0,
			duplicate_rate_pct: 0.0,
			p99_match_ms: 0,
		}),
	)
}
