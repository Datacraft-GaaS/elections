use axum::body::Body;
use axum::http::{Request, StatusCode};
use dispatch_api::{app, state::AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

fn check_in_body() -> Value {
	json!({
		"workerId": "11111111-1111-1111-1111-111111111111",
		"stationCode": "STA-01",
		"electionId": "22222222-2222-2222-2222-222222222222",
		"biometricHash": "0a1b2c3d4e5f",
		"gpsLat": -1.286389,
		"gpsLon": 36.817223,
		"gpsAccuracyM": 3.2,
		"signature": "deadbeef"
	})
}

#[tokio::test]
async fn check_in_persists_and_status_ping_is_retrievable() {
	let app = app(AppState::default());

	// Check-in.
	let check_in_resp = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/v1/dispatch/check-in")
				.header("content-type", "application/json")
				.body(Body::from(check_in_body().to_string()))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(check_in_resp.status(), StatusCode::CREATED);

	// Status ping for the same worker.
	let ping_resp = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/v1/dispatch/ping")
				.header("content-type", "application/json")
				.body(Body::from(
					json!({
						"workerId": "11111111-1111-1111-1111-111111111111",
						"stationCode": "STA-01",
						"status": "ON_STATION",
						"reason": null
					})
					.to_string(),
				))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(ping_resp.status(), StatusCode::OK);

	// Fetch the worker's status.
	let status_resp = app
		.oneshot(
			Request::builder()
				.uri("/v1/dispatch/workers/11111111-1111-1111-1111-111111111111/status")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(status_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_check_in_returns_400() {
	let app = app(AppState::default());
	let resp = app
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/v1/dispatch/check-in")
				.header("content-type", "application/json")
				.body(Body::from(json!({ "bogus": true }).to_string()))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn assignments_returns_list() {
	let app = app(AppState::default());
	let resp = app
		.oneshot(
			Request::builder()
				.uri("/v1/dispatch/assignments/11111111-1111-1111-1111-111111111111")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(resp.status(), StatusCode::OK);
}
