//! ABIS API library: router and shared types.

pub mod error;
pub mod idempotency;
pub mod middleware;
pub mod models;
pub mod openapi;
pub mod routes;
pub mod state;

use std::sync::Arc;

use axum::{
	Router,
	middleware as axum_middleware,
	routing::{get, patch, post},
};

use routes::{
	adjudication::{
		assign_adjudication, decide_adjudication, escalate_adjudication, list_adjudication,
		list_evidence, resolve_adjudication, upload_evidence,
	},
	admin::{list_vendors, update_policy, vendor_health_check},
	batch::submit_batch_dedup,
	enrolment::{
		create_enrolment_job, get_enrolment_job, get_quality_report, withdraw_enrolment_job,
	},
	health::{healthz, readyz},
	kit::{kit_enrol, kit_gallery_config, kit_verify},
	match_routes::{get_match_result, match_11, match_1n, match_batch},
	reports::{get_dedup_summary, get_enrolment_throughput, get_fairness, get_fmr_fnmr, get_reports},
};
use state::AppState;

/// Build the full axum router.
pub fn app(state: Arc<AppState>) -> Router {
	// v1 enrolment routes
	let enrolment = Router::new()
		.route("/jobs", post(create_enrolment_job))
		.route("/jobs/{id}", get(get_enrolment_job))
		.route("/jobs/{id}/withdraw", patch(withdraw_enrolment_job))
		.route("/jobs/{id}/quality-report", get(get_quality_report));

	// v1 match routes
	let match_r = Router::new()
		.route("/1n", post(match_1n))
		.route("/11", post(match_11))
		.route("/batch", post(match_batch))
		.route("/{request_id}", get(get_match_result));

	// v1 adjudication routes
	let adjudication = Router::new()
		.route("/cases", get(list_adjudication))
		.route("/cases/{id}/assign", post(assign_adjudication))
		.route("/cases/{id}/decide", post(decide_adjudication))
		.route("/cases/{id}/escalate", post(escalate_adjudication))
		.route("/cases/{id}/evidence", get(list_evidence).post(upload_evidence));

	// v1 kit routes
	let kit = Router::new()
		.route("/enrolment", post(kit_enrol))
		.route("/verify", post(kit_verify))
		.route("/gallery-config", get(kit_gallery_config));

	// v1 reports routes
	let reports = Router::new()
		.route("/fmr-fnmr", get(get_fmr_fnmr))
		.route("/dedup-summary", get(get_dedup_summary))
		.route("/fairness", get(get_fairness))
		.route("/enrolment-throughput", get(get_enrolment_throughput));

	// v1 admin routes
	let admin = Router::new()
		.route("/policies", post(update_policy))
		.route("/vendors", get(list_vendors))
		.route("/vendors/{id}/health-check", post(vendor_health_check));

	// v1 router
	let v1 = Router::new()
		.nest("/enrolment", enrolment)
		.nest("/match", match_r)
		.nest("/adjudication", adjudication)
		.nest("/kit", kit)
		.nest("/reports", reports)
		.nest("/admin", admin);

	// Legacy /api/v1/abis routes (backward compat)
	let legacy = Router::new()
		.route("/match/1n", post(match_1n))
		.route("/match/11", post(match_11))
		.route("/match/{request_id}", get(get_match_result))
		.route("/adjudication", get(list_adjudication))
		.route("/adjudication/{case_id}", patch(resolve_adjudication))
		.route("/batch/dedup", post(submit_batch_dedup))
		.route("/reports", get(get_reports));

	// Build the stateful API router first, then apply state (-> Router<()>).
	let api: Router = Router::new()
		.route("/healthz", get(healthz))
		.route("/readyz", get(readyz))
		.route("/openapi.json", get(openapi::serve_openapi))
		.nest("/v1", v1)
		.nest("/api/v1/abis", legacy)
		.layer(axum_middleware::from_fn(middleware::cosign_response))
		.with_state(state);

	api
}
