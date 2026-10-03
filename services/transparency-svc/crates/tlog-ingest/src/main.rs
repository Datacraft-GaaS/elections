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
use tlog_ingest::IngestConsumer;
use tlog_sequencer::Sequencer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://eel:eel@localhost/eel_tlog".to_string());
    let brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(32)
            .connect(&db_url)
            .await?,
    );
    let sequencer = Arc::new(Sequencer::new(pool.clone(), 50_000));
    let consumer = Arc::new(IngestConsumer::new(pool, sequencer, 1000));

    let topics = [
        "eel.voter-events",
        "eel.results-events",
        "eel.custody-events",
        "eel.credential-events",
        "eel.meta-events",
    ];
    let topic_refs: Vec<&str> = topics.iter().copied().collect();
    consumer.run(&brokers, "tlog-ingest", &topic_refs).await?;
    Ok(())
}
