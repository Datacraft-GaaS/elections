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

//! eel-adapter-satellite binary entry point.

use std::{env, net::SocketAddr, sync::Arc};
use satellite::{AppState, router};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();

	let state = Arc::new(AppState {
		base_url: env::var("SATELLITE_BASE_URL")
			.unwrap_or_else(|_| "https://api.example.go.ke".into()),
		api_key: env::var("SATELLITE_API_KEY").unwrap_or_default(),
		client: reqwest::Client::builder()
			.timeout(std::time::Duration::from_secs(10))
			.build()
			.unwrap(),
	});

	let addr: SocketAddr = env::var("SATELLITE_LISTEN_ADDR")
		.unwrap_or_else(|_| "0.0.0.0:8080".into())
		.parse()
		.unwrap();

	tracing::info!("eel-adapter-satellite listening on {}", addr);
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, router(state)).await.unwrap();
}
