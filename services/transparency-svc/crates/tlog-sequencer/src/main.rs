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
use tlog_sequencer::{signer::SthSigner, Sequencer, SthProducer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://eel:eel@localhost/eel_tlog".to_string());
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(32)
            .connect(&db_url)
            .await?,
    );
    // Signing backend from env: PKCS#11/softHSM by default; `EEL_STH_SIGNER=dev`
    // selects an explicit development Ed25519 key (refused without key material).
    let signer = SthSigner::from_env().expect("STH signer configuration failed");
    let producer = Arc::new(SthProducer::new(pool.clone(), signer));
    producer.clone().spawn_all();

    let _sequencer = Sequencer::new(pool, 50_000);
    tracing::info!("tlog-sequencer started");

    tokio::signal::ctrl_c().await?;
    Ok(())
}
