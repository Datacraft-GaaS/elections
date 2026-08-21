//! `tlog-admin bulk-ingest --topic <t> --from-offset <n>` — replay a Kafka topic into the log.

use anyhow::Result;

pub async fn run(topic: &str, from_offset: i64, _api_url: &str) -> Result<()> {
    tracing::info!(topic = %topic, from_offset = from_offset, "bulk-ingest started (stub)");
    println!(
        "bulk-ingest from offset {} on topic {} (stub)",
        from_offset, topic
    );
    Ok(())
}
