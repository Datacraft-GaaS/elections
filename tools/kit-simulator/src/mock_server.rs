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

use anyhow::Result;
use axum::{extract::Request, http::StatusCode, response::IntoResponse, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Simulated latency per endpoint family (ms, base).
fn latency_base_ms(path: &str) -> u64 {
	if path.contains("bvi") {
		15
	} else if path.contains("rts") {
		20
	} else if path.contains("portal") || path.contains("results") {
		8
	} else if path.contains("verify") {
		12
	} else if path.contains("heartbeat") {
		5
	} else {
		10
	}
}

async fn mock_handler(req: Request) -> impl IntoResponse {
	let path = req.uri().path().to_string();
	let base = latency_base_ms(&path);
	// Small jitter using subsecond nanos as a cheap source
	let jitter = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap_or_default()
		.subsec_nanos() as u64 % 30;
	tokio::time::sleep(std::time::Duration::from_millis(base + jitter)).await;
	(
		StatusCode::OK,
		axum::Json(serde_json::json!({
			"status": "ok",
			"mock": true,
			"path": path
		})),
	)
}

/// In-process mock HTTP server for dry-run mode.
pub struct MockServer {
	pub addr: SocketAddr,
	#[allow(dead_code)]
	shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl MockServer {
	/// Bind to 127.0.0.1:0, start serving, return immediately.
	pub async fn start() -> Result<Self> {
		let listener = TcpListener::bind("127.0.0.1:0").await?;
		let addr = listener.local_addr()?;
		let (tx, rx) = tokio::sync::oneshot::channel::<()>();

		let app = Router::new().fallback(mock_handler);

		tokio::spawn(async move {
			axum::serve(listener, app)
				.with_graceful_shutdown(async {
					let _ = rx.await;
				})
				.await
				.ok();
		});

		Ok(MockServer { addr, shutdown_tx: tx })
	}

	pub fn base_url(&self) -> String {
		format!("http://{}", self.addr)
	}

	/// Signal graceful shutdown.
	pub fn stop(self) {
		let _ = self.shutdown_tx.send(());
	}
}
