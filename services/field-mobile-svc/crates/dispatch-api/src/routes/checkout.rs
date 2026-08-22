use axum::{
	extract::State,
	http::StatusCode,
	Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::models::WorkerCheckOut;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckOutRequest {
	worker_id: Uuid,
	station_code: String,
	election_id: Uuid,
	gps_lat: f64,
	gps_lon: f64,
	signature: String,
}

pub async fn check_out(
	state: State<AppState>,
	Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
	let req: CheckOutRequest = match serde_json::from_value(body) {
		Ok(r) => r,
		Err(_) => {
			return (
				StatusCode::BAD_REQUEST,
				Json(serde_json::json!({ "error": "invalid check-out request" })),
			)
		}
	};

	// Compute hours worked from the worker's most recent check-in, if any.
	let check_in_at = state
		.check_ins
		.lock()
		.ok()
		.and_then(|store| {
			store
				.iter()
				.filter(|c| c.worker_id == req.worker_id)
				.max_by_key(|c| c.check_in_at)
				.map(|c| c.check_in_at)
		});
	let check_out_at = Utc::now();
	let hours_worked = match check_in_at {
		Some(start) => {
			let secs = check_out_at.signed_duration_since(start).num_seconds();
			(secs.max(0) as f32) / 3600.0
		}
		None => 0.0,
	};

	let check_out = WorkerCheckOut {
		id: Uuid::new_v4(),
		worker_id: req.worker_id,
		station_code: req.station_code,
		election_id: req.election_id,
		gps_lat: req.gps_lat,
		gps_lon: req.gps_lon,
		check_out_at,
		hours_worked,
		payroll_trigger_id: None,
		signature: req.signature,
	};

	let Ok(mut store) = state.check_outs.lock() else {
		return (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(serde_json::json!({ "error": "store unavailable" })),
		);
	};
	store.push(check_out.clone());

	(
		StatusCode::CREATED,
		Json(serde_json::to_value(check_out).unwrap_or_default()),
	)
}
