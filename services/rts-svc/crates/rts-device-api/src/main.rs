use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rts_device_api=info".parse().unwrap()),
        )
        .json()
        .init();

    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string());
    info!(http_addr, "rts-device-api starting");

    let app = rts_device_api::router::build_router();
    let listener = tokio::net::TcpListener::bind(&http_addr)
        .await
        .expect("bind failed");
    info!(http_addr, "rts-device-api listening");
    axum::serve(listener, app).await.expect("HTTP server failed");
}
