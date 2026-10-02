// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
