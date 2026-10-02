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
                .add_directive("ecc_api=info".parse().unwrap()),
        )
        .json()
        .init();
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8084".to_string());
    info!(http_addr, "ecc-api starting");

    let store = match env::var("DATABASE_URL") {
        Ok(url) => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(10)
                .connect(&url)
                .await
                .expect("database connect failed");
            ecc_api::store::Store::Pg(pool)
        }
        Err(_) => {
            info!("DATABASE_URL unset; using in-memory store");
            ecc_api::store::Store::Mem(Default::default())
        }
    };

    let app = ecc_api::router::build_router(store);
    let listener = tokio::net::TcpListener::bind(&http_addr).await.expect("bind failed");
    info!(http_addr, "ecc-api HTTP listening");
    axum::serve(listener, app).await.expect("HTTP server failed");
}
