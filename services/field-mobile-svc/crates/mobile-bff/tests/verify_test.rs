use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use mobile_bff::{router::build_router, sse::ShardedBroadcast};
use tower::ServiceExt;

fn make_router() -> axum::Router {
	let broadcast = ShardedBroadcast::new();
	mobile_bff::router::build_router_with_state(broadcast)
}

#[tokio::test]
// #[trace("FR-M13-001", "AC-M13-01")]
async fn health_returns_200() {
	let app = make_router();
	let req = Request::builder()
		.uri("/health")
		.body(axum::body::Body::empty())
		.unwrap();
	let resp = app.oneshot(req).await.unwrap();
	assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
// #[trace("FR-M13-025", "AC-M13-03")]
async fn results_endpoint_returns_200() {
	let app = make_router();
	let req = Request::builder()
		.uri("/verify/v1/results")
		.body(axum::body::Body::empty())
		.unwrap();
	let resp = app.oneshot(req).await.unwrap();
	assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
// #[trace("FR-M13-030", "AC-M13-05")]
async fn results_by_scope_returns_200() {
	let app = make_router();
	let req = Request::builder()
		.uri("/verify/v1/results/constituency/001")
		.body(axum::body::Body::empty())
		.unwrap();
	let resp = app.oneshot(req).await.unwrap();
	assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
// #[trace("FR-M13-022", "AC-M13-06")]
async fn clone_report_returns_200() {
	let app = make_router();
	let body = serde_json::json!({
		"app_id": "com.eel.citizenverify",
		"package_name": "com.eel.citizenverify",
		"reported_signature": "deadbeef",
		"platform": "android"
	});
	let req = Request::builder()
		.uri("/verify/v1/clone-report")
		.method("POST")
		.header("content-type", "application/json")
		.body(axum::body::Body::from(body.to_string()))
		.unwrap();
	let resp = app.oneshot(req).await.unwrap();
	assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
// #[trace("FR-M13-001", "EC-SCL-01")]
async fn sse_shard_index_is_deterministic() {
	use mobile_bff::sse::station_shard;
	let s1 = station_shard("KE-001");
	let s2 = station_shard("KE-001");
	assert_eq!(s1, s2);
	assert!(s1 < 16);
}

#[tokio::test]
// #[trace("FR-M13-032", "AC-M13-05")]
async fn grievance_endpoint_returns_200() {
	let app = make_router();
	let body = serde_json::json!({
		"channel": "flutter",
		"description": "Observed irregularity at station",
		"station_code": "KE-NBI-001",
		"idempotency_key": "test-key-001"
	});
	let req = Request::builder()
		.uri("/verify/v1/grievances")
		.method("POST")
		.header("content-type", "application/json")
		.body(axum::body::Body::from(body.to_string()))
		.unwrap();
	let resp = app.oneshot(req).await.unwrap();
	assert_eq!(resp.status(), StatusCode::OK);
}
