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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("verifier_api=info".parse().unwrap()),
        )
        .json()
        .init();
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8131".to_string());
    info!(http_addr, "verifier-api starting");
    // Three secrets, none of which belong in a repository. Each is provider- or
    // telco-issued and read from configuration; with any of them absent the
    // corresponding endpoint reports it cannot verify rather than claiming a check
    // it did not perform.
    let mut store = verifier_api::store::VerifierStore::new();
    if let Ok(v) = env::var("WHATSAPP_VERIFY_TOKEN") {
        if !v.trim().is_empty() {
            store = store.with_whatsapp_verify_token(v);
        }
    }
    if let Ok(v) = env::var("WHATSAPP_APP_SECRET") {
        if !v.trim().is_empty() {
            store = store.with_whatsapp_app_secret(v);
        }
    }
    if let Ok(v) = env::var("USSD_WEBHOOK_SECRET") {
        if !v.trim().is_empty() {
            store = store.with_ussd_webhook_secret(v);
        }
    }
    let app = verifier_api::http::router(store.shared());
    let listener = tokio::net::TcpListener::bind(&http_addr)
        .await
        .expect("bind failed");
    info!(http_addr, "verifier-api HTTP listening");
    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}
