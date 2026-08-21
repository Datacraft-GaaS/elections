use axum::{
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::state::AppState;

pub async fn list_tickets(_state: State<AppState>) -> (StatusCode, Json<Value>) {
	(
		StatusCode::NOT_IMPLEMENTED,
		Json(serde_json::json!({"message": "not implemented"})),
	)
}

pub async fn create_ticket(
	_state: State<AppState>,
	Json(_body): Json<Value>,
) -> (StatusCode, Json<Value>) {
	(
		StatusCode::NOT_IMPLEMENTED,
		Json(serde_json::json!({"message": "not implemented"})),
	)
}

pub async fn get_ticket(
	_state: State<AppState>,
	Path(_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
	(
		StatusCode::NOT_IMPLEMENTED,
		Json(serde_json::json!({"message": "not implemented"})),
	)
}
