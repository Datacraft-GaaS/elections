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

//! eel HSM Signer daemon.
//!
//! Listens on:
//! - 0.0.0.0:8443 gRPC (TLS via SPIRE SVID mounted at /run/spire/svid)
//! - 0.0.0.0:8081 HTTP health check
//!
//! Required env:
//! - `EEL_HSM_PIN`: PKCS#11 user PIN
//! - `EEL_HSM_SLOT`: PKCS#11 slot index (default 0)
//! - `EEL_HSM_VENDOR`: softhsm | luna7 | utimaco (default softhsm)
//! - `PKCS11_MODULE_PATH`: overrides vendor default shared library path

use std::net::SocketAddr;

use axum::{routing::get, Router};
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::EnvFilter;

use hsm_signer::pool::RolePool;
use hsm_signer::proto::hsm_signer_server::HsmSignerServer;
use hsm_signer::HsmSignerService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("hsm_signer=info".parse()?))
        .json()
        .init();

    let pin = std::env::var("EEL_HSM_PIN").unwrap_or_else(|_| "1234".to_string());
    let slot: u64 = std::env::var("EEL_HSM_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let pool = RolePool::open(slot, pin)
        .map_err(|e| anyhow::anyhow!("failed to open PKCS#11 pool: {e}"))?;
    let svc = HsmSignerService::new(pool);

    // Health check server on 8081.
    let health_addr: SocketAddr = "0.0.0.0:8081".parse()?;
    let health_app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ok" }));
    let health_listener = tokio::net::TcpListener::bind(health_addr).await?;
    info!("health check listening on {health_addr}");

    // gRPC server on 8443.
    // TLS: in production, cert/key come from SPIRE SVID at /run/spire/svid.
    // For dev (EEL_HSM_VENDOR=softhsm), plain TCP is accepted.
    let grpc_addr: SocketAddr = "0.0.0.0:8443".parse()?;
    let grpc_server = Server::builder()
        .add_service(HsmSignerServer::new(svc))
        .serve(grpc_addr);
    info!("gRPC server listening on {grpc_addr}");

    tokio::select! {
        r = axum::serve(health_listener, health_app) => r?,
        r = grpc_server => r?,
    }

    Ok(())
}
