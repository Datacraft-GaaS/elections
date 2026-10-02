// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
///
/// **This previously returned 200 with `data: null` for any constituency**,
/// described as a relay from a service not in this workspace. A constituency with
/// no recorded tokens is a 404.
async fn results_by_scope_requires_recorded_tokens() {
	let app = make_router();
	let req = Request::builder()
		.uri("/verify/v1/results/constituency/001")
		.body(axum::body::Body::empty())
		.unwrap();
	let resp = app.oneshot(req).await.unwrap();
	assert_eq!(
		resp.status(),
		StatusCode::NOT_FOUND,
		"no tokens have been recorded for this constituency"
	);
}

#[tokio::test]
// #[trace("FR-M13-022", "AC-M13-06")]
///
/// A filed clone report is a new record, so `201 Created` is the honest status.
/// It previously returned 200 for a report that was minted and stored nowhere.
async fn clone_report_is_created() {
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
	assert_eq!(resp.status(), StatusCode::CREATED);
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
