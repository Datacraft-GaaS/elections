use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("printing=info".parse().unwrap()),
        )
        .json()
        .init();
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8111".to_string());
    info!(http_addr, "printing-api starting");
    let state = printing::http::AppState::new_shared();
    let app = printing::http::router(state);
    let listener = tokio::net::TcpListener::bind(&http_addr).await.expect("bind failed");
    info!(http_addr, "printing-api HTTP listening");
    axum::serve(listener, app).await.expect("HTTP server failed");
}
