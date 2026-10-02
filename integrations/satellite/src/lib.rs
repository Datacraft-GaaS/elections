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

//! eel-adapter-satellite: Starlink/VSAT fallback bearer adapter for edge kits
//!
//! Wire: Starlink gRPC API + VSAT AT-command bridge; satellite link status, bandwidth, latency metrics.
#![warn(missing_docs)]

use axum::{extract::State, http::StatusCode, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
	pub base_url: String,
	pub api_key: String,
	pub client: reqwest::Client,
}

#[derive(Deserialize)]
pub struct AdapterRequest {
	pub request_id: String,
}

#[derive(Serialize)]
pub struct AdapterResponse {
	pub request_id: String,
	pub ok: bool,
	pub message: String,
}

pub async fn health() -> Json<serde_json::Value> {
	Json(serde_json::json!({"status": "ok"}))
}

pub async fn handle_satellite(
	State(state): State<Arc<AppState>>,
	Json(req): Json<AdapterRequest>,
) -> Result<Json<AdapterResponse>, StatusCode> {
	let resp = state
		.client
		.post(format!("{}/satellite/status", state.base_url))
		.header("X-API-Key", &state.api_key)
		.json(&serde_json::json!({"request_id": req.request_id}))
		.send()
		.await
		.map_err(|_| StatusCode::BAD_GATEWAY)?;
	if !resp.status().is_success() {
		return Err(StatusCode::BAD_GATEWAY);
	}
	Ok(Json(AdapterResponse {
		request_id: req.request_id,
		ok: true,
		message: "ok".into(),
	}))
}

pub fn router(state: Arc<AppState>) -> Router {
	Router::new()
		.route("/satellite/status", post(handle_satellite))
		.route("/health", get(health))
		.with_state(state)
}
