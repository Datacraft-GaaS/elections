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

//! ABIS API server entrypoint.
//!
//! Ports:
//!   REST  -> EEL_ABIS_REST_ADDR  (default: 0.0.0.0:8080)
//!   Health-> EEL_ABIS_HEALTH_ADDR (default: 0.0.0.0:8081)

use std::sync::Arc;

use abis_api::{app, state::AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState::new());
    let router = app(state);

    let rest_addr = std::env::var("EEL_ABIS_REST_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());

    let listener = tokio::net::TcpListener::bind(&rest_addr)
        .await
        .unwrap_or_else(|e| panic!("bind {rest_addr}: {e}"));

    tracing::info!(addr = rest_addr, "abis-api REST server starting");
    axum::serve(listener, router).await.expect("serve");
}
