use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("rts_api=info".parse().unwrap()),
    ).json().init();
    let _db_url = env::var("DB_URL").unwrap_or_else(|_| "postgres://localhost/eel".to_string());
    let http_addr = "0.0.0.0:8080";
    info!(http_addr, "rts-api starting");
    let app = rts_api::router::build_router();
    let listener = tokio::net::TcpListener::bind(http_addr).await.expect("bind failed");
    info!(http_addr, "rts-api HTTP listening");
    axum::serve(listener, app).await.expect("HTTP server failed");
}
