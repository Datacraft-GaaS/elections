use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let addr: SocketAddr = std::env::var("HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8090".to_owned())
        .parse()?;

    tracing::info!(%addr, "budget-svc starting");

    let router = budget::routes::router();
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("HTTP listening on {addr}");
    axum::serve(listener, router).await?;

    Ok(())
}
