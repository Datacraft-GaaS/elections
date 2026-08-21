//! eel-adapter-ipfs binary entry point.

use std::{env, net::SocketAddr, sync::Arc};
use ipfs::{AppState, router};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env())
		.init();

	let state = Arc::new(AppState {
		base_url: env::var("IPFS_BASE_URL")
			.unwrap_or_else(|_| "https://api.example.go.ke".into()),
		api_key: env::var("IPFS_API_KEY").unwrap_or_default(),
		client: reqwest::Client::builder()
			.timeout(std::time::Duration::from_secs(10))
			.build()
			.unwrap(),
	});

	let addr: SocketAddr = env::var("IPFS_LISTEN_ADDR")
		.unwrap_or_else(|_| "0.0.0.0:8080".into())
		.parse()
		.unwrap();

	tracing::info!("eel-adapter-ipfs listening on {}", addr);
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, router(state)).await.unwrap();
}
