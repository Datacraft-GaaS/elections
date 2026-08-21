use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tlog_ingest::IngestConsumer;
use tlog_sequencer::Sequencer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://eel:eel@localhost/eel_tlog".to_string());
    let brokers = std::env::var("KAFKA_BROKERS")
        .unwrap_or_else(|_| "localhost:9092".to_string());

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
