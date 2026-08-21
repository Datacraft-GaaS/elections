use dispatch_api::{app, state::AppState};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();
	let state = AppState::default();
	let addr: SocketAddr = "0.0.0.0:8083".parse().unwrap();
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, app(state)).await.unwrap();
}
