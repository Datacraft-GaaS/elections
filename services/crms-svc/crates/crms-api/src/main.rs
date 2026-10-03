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

//! crms-api server entrypoint.
//!
//! Composes the CRMS route table with the shared `eel-api-server-rs` builder,
//! which layers auth (JWKS), load-shedding, idempotency, content negotiation,
//! request-id, and OTel tracing over the application routes.

use std::sync::Arc;

use eel_api_server_rs::ApiServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let state = Arc::new(crms_api::AppState::new());
    let app = crms_api::build_router(state);

    let http_addr = std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let server = ApiServer::builder()
        .otel_service_name("crms-api")
        .bind_addr(http_addr)
        .build()?;

    server.serve(app).await?;
    Ok(())
}
