use axum::{
	extract::State,
	http::StatusCode,
	Json,
};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::models::{CheckInRequest, WorkerCheckIn};
use crate::state::AppState;

pub async fn check_in(
	state: State<AppState>,
	Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
	let req: CheckInRequest = match serde_json::from_value(body) {
		Ok(r) => r,
		Err(_) => {
			return (
				StatusCode::BAD_REQUEST,
				Json(serde_json::json!({ "error": "invalid check-in request" })),
			)
		}
	};

	let check_in = WorkerCheckIn {
		id: Uuid::new_v4(),
		worker_id: req.worker_id,
		station_code: req.station_code,
		election_id: req.election_id,
		biometric_hash: req.biometric_hash,
		gps_lat: req.gps_lat,
		gps_lon: req.gps_lon,
		gps_accuracy_m: req.gps_accuracy_m,
		check_in_at: Utc::now(),
		signature: req.signature,
	};

	let Ok(mut store) = state.check_ins.lock() else {
		return (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(serde_json::json!({ "error": "store unavailable" })),
		);
	};
	store.push(check_in.clone());

	(
		StatusCode::CREATED,
		Json(serde_json::to_value(check_in).unwrap_or_default()),
	)
}
