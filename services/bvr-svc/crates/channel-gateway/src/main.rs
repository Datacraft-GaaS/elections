use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("channel_gateway=info".parse().unwrap()),
        )
        .json()
        .init();
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8132".to_string());
    info!(http_addr, "channel-gateway starting");
    let state = std::sync::Arc::new(match std::env::var("WHATSAPP_APP_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => {
            channel_gateway::store::AppState::new().with_whatsapp_secret(secret)
        }
        _ => channel_gateway::store::AppState::new(),
    });
    let app = channel_gateway::http::router(state);
    let listener = tokio::net::TcpListener::bind(&http_addr).await.expect("bind failed");
    info!(http_addr, "channel-gateway HTTP listening");
    axum::serve(listener, app).await.expect("HTTP server failed");
}
