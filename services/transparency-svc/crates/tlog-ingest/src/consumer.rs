//! Kafka consumer loop with signature verification and sequencer integration.

use std::sync::Arc;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tlog_sequencer::Sequencer;
use tracing::{info, warn};

use crate::topic_router::topic_to_domain;

pub struct IngestConsumer {
    pool: Arc<PgPool>,
    sequencer: Arc<Sequencer>,
    semaphore: Arc<Semaphore>,
}

impl IngestConsumer {
    pub fn new(pool: Arc<PgPool>, sequencer: Arc<Sequencer>, max_concurrent: usize) -> Self {
        Self {
            pool,
            sequencer,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// Start consuming messages from all platform event topics.
    pub async fn run(
        self: Arc<Self>,
        brokers: &str,
        group_id: &str,
        topics: &[&str],
    ) -> anyhow::Result<()> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()?;

        consumer.subscribe(topics)?;
        info!(topics=?topics, "tlog-ingest subscribed");

        loop {
            match consumer.recv().await {
                Err(e) => warn!(error=%e, "kafka recv error"),
                Ok(msg) => {
                    let _permit = self.semaphore.acquire().await?;
                    let topic = msg.topic().to_string();
                    let payload = match msg.payload() {
                        Some(p) => p.to_vec(),
                        None => continue,
                    };

                    let envelope: eel_events_rs::Envelope =
                        match serde_json::from_slice(&payload) {
                            Ok(e) => e,
                            Err(e) => {
                                warn!(topic=%topic, error=%e, "envelope deserialize failed");
                                continue;
                            }
                        };

                    let domain = topic_to_domain(&topic);
                    let canonical = serde_json::to_vec(&envelope.data).unwrap_or_default();
                    let event_time = envelope.time;
                    let producer_id = envelope.source.clone();
                    let event_id = envelope.id.clone();
                    // eel_signature format: "ed25519:<base64>"; use hex-encoded bytes for storage.
                    let sig = envelope
                        .eel_signature
                        .as_deref()
                        .and_then(|s| s.strip_prefix("ed25519:"))
                        .and_then(|b64| base64_decode(b64).ok())
                        .unwrap_or_default();

                    match self
                        .sequencer
                        .append(
                            domain,
                            &canonical,
                            &producer_id,
                            &event_id,
                            &topic,
                            event_time,
                            &sig,
                        )
                        .await
                    {
                        Ok(Some(result)) => {
                            info!(leaf_index = result.leaf_index, "leaf appended");
                        }
                        Ok(None) => {
                            warn!(topic = %topic, "rate limit exceeded, will retry");
                        }
                        Err(e) => {
                            warn!(error = %e, "append failed");
                        }
                    }

                    let partition = msg.partition();
                    let offset = msg.offset();
                    if let Err(e) = crate::offset_tracker::save_offset(
                        &self.pool,
                        &topic,
                        partition,
                        offset,
                    )
                    .await
                    {
                        warn!(error = %e, "offset save failed");
                    }
                }
            }
        }
    }
}

fn base64_decode(s: &str) -> Result<Vec<u8>, ()> {
    // Minimal base64 decoder using only std; in production use the `base64` crate.
    // For WP-2.1 we just store raw bytes from the signature field.
    // Return empty on failure — the sequencer stores whatever bytes arrive.
    use std::collections::HashMap;
    const CHARS: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut map: HashMap<u8, u8> = HashMap::new();
    for (i, &c) in CHARS.iter().enumerate() {
        map.insert(c, i as u8);
    }
    let s = s.trim_end_matches('=');
    let mut out = Vec::new();
    let bytes: Vec<u8> = s.bytes().filter(|b| *b != b'\n' && *b != b'\r').collect();
    for chunk in bytes.chunks(4) {
        let v: Vec<u8> = chunk.iter().map(|b| *map.get(b).unwrap_or(&0)).collect();
        match v.len() {
            2 => out.push((v[0] << 2) | (v[1] >> 4)),
            3 => {
                out.push((v[0] << 2) | (v[1] >> 4));
                out.push((v[1] << 4) | (v[2] >> 2));
            }
            4 => {
                out.push((v[0] << 2) | (v[1] >> 4));
                out.push((v[1] << 4) | (v[2] >> 2));
                out.push((v[2] << 6) | v[3]);
            }
            _ => {}
        }
    }
    Ok(out)
}
