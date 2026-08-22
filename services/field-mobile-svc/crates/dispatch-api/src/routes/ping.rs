use axum::{
	extract::State,
	http::StatusCode,
	Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::models::{WorkerStatus, WorkerStatusPing};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusPingRequest {
	worker_id: Uuid,
	station_code: String,
	status: WorkerStatus,
	reason: Option<String>,
}

pub async fn status_ping(
	state: State<AppState>,
	Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
	let req: StatusPingRequest = match serde_json::from_value(body) {
		Ok(r) => r,
		Err(_) => {
			return (
				StatusCode::BAD_REQUEST,
				Json(serde_json::json!({ "error": "invalid status ping" })),
			)
		}
	};

	let ping = WorkerStatusPing {
		id: Uuid::new_v4(),
		worker_id: req.worker_id,
		station_code: req.station_code,
		status: req.status,
		reason: req.reason,
		pinged_at: Utc::now(),
	};

	let Ok(mut store) = state.pings.lock() else {
		return (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(serde_json::json!({ "error": "store unavailable" })),
		);
	};
	store.push(ping.clone());

	(
		StatusCode::OK,
		Json(serde_json::to_value(ping).unwrap_or_default()),
	)
}
