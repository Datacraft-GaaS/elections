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

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use support_api::{app, state::AppState};
use tower::ServiceExt;

#[tokio::test]
async fn create_and_fetch_ticket() {
	let app = app(AppState::default());

	let create = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/v1/support/tickets")
				.header("content-type", "application/json")
				.body(Body::from(
					json!({
						"stationCode": "STA-01",
						"kitSerial": "EEL-KIT-0001",
						"severity": "SEV2",
						"title": "Printer jammed",
						"description": "Thermal printer not feeding paper"
					})
					.to_string(),
				))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(create.status(), StatusCode::CREATED);

	let body_bytes = axum::body::to_bytes(create.into_body(), usize::MAX)
		.await
		.unwrap();
	let created: Value = serde_json::from_slice(&body_bytes).unwrap();
	let id = created["id"].as_str().unwrap().to_string();

	let list = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/v1/support/tickets")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(list.status(), StatusCode::OK);

	let get = app
		.oneshot(
			Request::builder()
				.uri(format!("/v1/support/tickets/{id}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(get.status(), StatusCode::OK);
}

#[tokio::test]
async fn warranty_claim_returns_created() {
	let app = app(AppState::default());
	let resp = app
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/v1/support/warranty")
				.header("content-type", "application/json")
				.body(Body::from(
					json!({
						"kitSerial": "EEL-KIT-0001",
						"oemCode": "OEM-42",
						"faultDescription": "Reader unresponsive",
						"engineerId": "33333333-3333-3333-3333-333333333333"
					})
					.to_string(),
				))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(resp.status(), StatusCode::CREATED);
}
