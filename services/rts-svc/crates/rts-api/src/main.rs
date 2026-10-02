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
    let state = std::sync::Arc::new(rts_api::store::AppState::default());
    let app = rts_api::router::build_router(state);
    let listener = tokio::net::TcpListener::bind(http_addr).await.expect("bind failed");
    info!(http_addr, "rts-api HTTP listening");
    axum::serve(listener, app).await.expect("HTTP server failed");
}
