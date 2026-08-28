use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".parse().unwrap()),
        )
        .json()
        .init();
    let addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8090".to_string());
    info!(addr, "ai-platform-api starting (store-only build; HTTP surface pending)");

    // Minimal liveness listener so orchestrators can probe the binary.
    let app = axum::Router::new().route("/healthz", axum::routing::get(|| async { "ok" }));
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}
