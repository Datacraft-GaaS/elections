use axum::{
	routing::{get, post},
	Router,
};

use crate::state::AppState;

pub mod diagnostics;
pub mod tickets;
pub mod warranty;

pub fn router(state: AppState) -> Router {
	Router::new()
		.route(
			"/healthz",
			get(|| async { axum::Json(serde_json::json!({"status":"ok"})) }),
		)
		.route(
			"/v1/support/tickets",
			get(tickets::list_tickets).post(tickets::create_ticket),
		)
		.route("/v1/support/tickets/{id}", get(tickets::get_ticket))
		.route(
			"/v1/support/tickets/{id}/diagnostics",
			post(diagnostics::run_diagnostic),
		)
		.route("/v1/support/warranty", post(warranty::submit_claim))
		.with_state(state)
}
