use axum::{
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::state::AppState;

pub async fn get_assignments(
	_state: State<AppState>,
	Path(_worker_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
	(
		StatusCode::NOT_IMPLEMENTED,
		Json(serde_json::json!({"message": "not implemented"})),
	)
}
