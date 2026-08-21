//! payroll service entry point (WP-5.6).

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = payroll::routes::router();

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    tracing::info!("payroll service listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
