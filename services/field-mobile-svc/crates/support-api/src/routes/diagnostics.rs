use axum::{
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::state::AppState;

pub async fn run_diagnostic(
	_state: State<AppState>,
	Path(_ticket_id): Path<Uuid>,
	Json(_body): Json<Value>,
) -> (StatusCode, Json<Value>) {
	(
		StatusCode::NOT_IMPLEMENTED,
		Json(serde_json::json!({"message": "not implemented"})),
	)
}
