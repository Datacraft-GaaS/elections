use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("eel_governance_legal_audit=info".parse().unwrap()),
        )
        .json()
        .init();
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8093".to_string());
    info!(http_addr, "eel-governance-legal-audit starting");
    let state = eel_governance_legal_audit::AppState::new();
    let app = eel_governance_legal_audit::router::build_router(state);
    let listener = tokio::net::TcpListener::bind(&http_addr).await.expect("bind failed");
    info!(http_addr, "eel-governance-legal-audit HTTP listening");
    axum::serve(listener, app).await.expect("HTTP server failed");
}
