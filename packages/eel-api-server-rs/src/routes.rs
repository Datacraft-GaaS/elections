use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

pub fn health_router() -> Router {
	Router::new()
		.route("/healthz", get(healthz))
		.route("/readyz", get(readyz))
		.route("/metrics", get(metrics_handler))
}

async fn healthz() -> Json<Value> {
	Json(json!({ "status": "ok" }))
}

async fn readyz() -> Json<Value> {
	Json(json!({ "status": "ready" }))
}

async fn metrics_handler() -> String {
	use prometheus::{Encoder, TextEncoder};
	let encoder = TextEncoder::new();
	let metric_families = prometheus::gather();
	let mut buf = Vec::new();
	encoder.encode(&metric_families, &mut buf).unwrap_or(());
	String::from_utf8(buf).unwrap_or_default()
}
