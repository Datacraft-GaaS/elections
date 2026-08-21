use axum::{
	extract::State,
	http::StatusCode,
	Json,
};
use serde_json::Value;

use crate::state::AppState;

pub async fn status_ping(
	_state: State<AppState>,
	Json(_body): Json<Value>,
) -> (StatusCode, Json<Value>) {
	(
		StatusCode::NOT_IMPLEMENTED,
		Json(serde_json::json!({"message": "not implemented"})),
	)
}
