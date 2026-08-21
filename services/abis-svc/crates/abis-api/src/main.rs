//! ABIS API server entrypoint.

use std::sync::Arc;

use abis_api::{app, state::AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState::new());
    let router = app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("bind 0.0.0.0:8080");

    tracing::info!("abis-api listening on 0.0.0.0:8080");
    axum::serve(listener, router).await.expect("serve");
}
