use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let http_addr: SocketAddr = std::env::var("HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
        .parse()?;

    let grpc_addr: SocketAddr = std::env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9090".to_owned())
        .parse()?;

    tracing::info!(%http_addr, %grpc_addr, "boundary-svc starting");

    // Spawn gRPC server (Geofence.Check).
    let store = boundary_geofence::server::demo_store();
    let geofence_svc = boundary_geofence::server::GeofenceService::new(store);
    let grpc_server = tonic::transport::Server::builder()
        .add_service(geofence_svc.into_server())
        .serve(grpc_addr);

    // Spawn background workers.
    tokio::spawn(boundary_worker::run_all_workers());

    // Spawn gRPC in background; serve HTTP in foreground.
    tokio::spawn(async move {
        if let Err(e) = grpc_server.await {
            tracing::error!(error = %e, "gRPC server error");
        }
    });

    let router = boundary_api::routes::router();
    let listener = TcpListener::bind(http_addr).await?;
    tracing::info!("HTTP listening on {http_addr}");
    axum::serve(listener, router).await?;

    Ok(())
}
