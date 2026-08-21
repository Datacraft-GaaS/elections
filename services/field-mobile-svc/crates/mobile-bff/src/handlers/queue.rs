use axum::{Json, extract::Path};
use serde_json::Value;
use tracing::info;

pub async fn get_queue_snapshot(
	Path(station): Path<String>,
) -> Json<Value> {
	info!(station, "queue.get_queue_snapshot");
	Json(serde_json::json!({
		"station": station,
		"queue_length": 0,
		"wait_estimate_minutes": 0,
		"note": "queue snapshot relay from M19"
	}))
}
