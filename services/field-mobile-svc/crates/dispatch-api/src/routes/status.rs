use axum::{
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::state::AppState;

pub async fn get_status(
	state: State<AppState>,
	Path(worker_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
	let Ok(store) = state.pings.lock() else {
		return (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(serde_json::json!({ "error": "store unavailable" })),
		);
	};

	match store.iter().filter(|p| p.worker_id == worker_id).max_by_key(|p| p.pinged_at) {
		Some(ping) => (
			StatusCode::OK,
			Json(serde_json::to_value(ping).unwrap_or_default()),
		),
		None => (
			StatusCode::NOT_FOUND,
			Json(serde_json::json!({ "error": "no status on record for worker" })),
		),
	}
}
