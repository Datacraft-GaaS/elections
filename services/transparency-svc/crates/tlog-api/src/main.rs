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

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tlog_api::{grpc, rest, ApiState};
use tlog_sequencer::Sequencer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://eel:eel@localhost/eel_tlog".to_string());

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(64)
            .connect(&db_url)
            .await?,
    );
    let sequencer = Arc::new(Sequencer::new(pool.clone(), 50_000));
    let state = ApiState { pool, sequencer };

    let rest_app = rest::router().with_state(state.clone());
    let grpc_svc = grpc::service(state);

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let grpc_addr =
        std::env::var("GRPC_ADDR").unwrap_or_else(|_| "0.0.0.0:50051".to_string());

    // REST (axum) and gRPC (tonic) share one ApiState and are served
    // concurrently; either address failing to bind aborts startup.
    let rest_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        tracing::info!(addr = %addr, "tlog-api REST listening");
        axum::serve(listener, rest_app).await?;
        anyhow::Ok(())
    });

    let grpc_task = tokio::spawn(async move {
        tracing::info!(addr = %grpc_addr, "tlog-api gRPC listening");
        tonic::transport::Server::builder()
            .add_service(grpc_svc)
            .serve(grpc_addr.parse()?)
            .await?;
        anyhow::Ok(())
    });

    let (rest_res, grpc_res) = tokio::join!(rest_task, grpc_task);
    rest_res??;
    grpc_res??;
    Ok(())
}
