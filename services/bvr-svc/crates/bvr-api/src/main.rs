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
    tracing_subscriber::fmt::init();
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    // The register is the `voters` table (migration V1), so the voter, status,
    // Merkle and SAR handlers need a database to answer honestly. With
    // DATABASE_URL unset they fall back to an in-memory register, which is
    // announced here because a register held only in process memory is not a
    // register. This mirrors credential-api.
    let store = match env::var("DATABASE_URL") {
        Ok(url) => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(10)
                .connect(&url)
                .await
                .expect("database connect failed");
            bvr_api::store::BvrStore::from_pool(pool)
        }
        Err(_) => {
            info!("DATABASE_URL unset; using in-memory register — prod must set postgres");
            bvr_api::store::BvrStore::in_memory()
        }
    };
    let state = bvr_api::state::AppState::new(
        env::var("IPRS_URL").unwrap_or_else(|_| "http://localhost:8090".to_string()),
        store,
    );
    let app = bvr_api::app(state);
    let listener = tokio::net::TcpListener::bind(&http_addr).await.unwrap();
    info!(http_addr, "bvr-api listening");
    axum::serve(listener, app).await.unwrap();
}
