use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("custody=info".parse().unwrap()),
        )
        .json()
        .init();
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8110".to_string());
    info!(http_addr, "custody-api starting");
    let state = custody::state::AppState::new();
    let app = custody::router::router(state);
    let listener = tokio::net::TcpListener::bind(&http_addr).await.expect("bind failed");
    info!(http_addr, "custody-api HTTP listening");
    axum::serve(listener, app).await.expect("HTTP server failed");
}
