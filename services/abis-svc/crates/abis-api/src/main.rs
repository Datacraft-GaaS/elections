//! ABIS API server entrypoint.
//!
//! Ports:
//!   REST  -> EEL_ABIS_REST_ADDR  (default: 0.0.0.0:8080)
//!   Health-> EEL_ABIS_HEALTH_ADDR (default: 0.0.0.0:8081)

use std::sync::Arc;

use abis_api::{app, state::AppState};

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	let state = Arc::new(AppState::new());
	let router = app(state);

	let rest_addr = std::env::var("EEL_ABIS_REST_ADDR")
		.unwrap_or_else(|_| "0.0.0.0:8080".into());

	let listener = tokio::net::TcpListener::bind(&rest_addr)
		.await
		.unwrap_or_else(|e| panic!("bind {rest_addr}: {e}"));

	tracing::info!(addr = rest_addr, "abis-api REST server starting");
	axum::serve(listener, router).await.expect("serve");
}
