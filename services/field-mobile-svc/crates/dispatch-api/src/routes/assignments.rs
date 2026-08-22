use axum::{
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::state::AppState;

pub async fn get_assignments(
	state: State<AppState>,
	Path(worker_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
	let Ok(store) = state.assignments.lock() else {
		return (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(serde_json::json!({ "error": "store unavailable" })),
		);
	};

	let assignments: Vec<_> = store
		.iter()
		.filter(|a| a.worker_id == worker_id)
		.cloned()
		.collect();

	(
		StatusCode::OK,
		Json(serde_json::to_value(assignments).unwrap_or_default()),
	)
}
