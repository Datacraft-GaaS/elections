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

use std::collections::BTreeMap;

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use tracing::{info_span, warn, Instrument};

use crate::{envelope::Envelope, error::EventError, spiffe::SpiffeKeyProvider};

// ---------------------------------------------------------------------------
// EventConsumer trait
// ---------------------------------------------------------------------------

/// Application-level event handler. Implementations receive fully-verified,
/// deduplicated events in causal order.
#[async_trait]
pub trait EventConsumer: Send + Sync {
    async fn handle(&self, envelope: Envelope) -> Result<(), EventError>;
}

// ---------------------------------------------------------------------------
// Signature verification
// ---------------------------------------------------------------------------

/// Verify `eel_signature` on an envelope. Returns `Err(EventError::TamperDetected)`
/// if the signature is missing, malformed, or invalid.
pub fn verify_signature(
    envelope: &Envelope,
    provider: &dyn SpiffeKeyProvider,
) -> Result<(), EventError> {
    let sig_str = envelope
        .eel_signature
        .as_deref()
        .ok_or(EventError::TamperDetected)?;

    let b64 = sig_str
        .strip_prefix("ed25519:")
        .ok_or(EventError::TamperDetected)?;

    let sig_bytes: [u8; 64] = STANDARD
        .decode(b64)
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or(EventError::TamperDetected)?;

    let sig = eel_crypto_rs::ed25519::signature_from_bytes(&sig_bytes)
        .map_err(|_| EventError::TamperDetected)?;

    let canonical = envelope.canonical_bytes_for_signing()?;

    eel_crypto_rs::ed25519::verify(&provider.signing_key().verifying_key(), &canonical, &sig)
        .map_err(|_| EventError::TamperDetected)
}

// ---------------------------------------------------------------------------
// Gap buffer: in-order delivery within a stream
// ---------------------------------------------------------------------------

/// Buffers out-of-order events within a stream and emits runs of sequential
/// events when gaps are filled.
///
/// Sequence numbers are expected to start at 0 and increment by 1.
#[derive(Default)]
pub struct GapBuffer {
    pending: BTreeMap<String, BTreeMap<u64, Envelope>>,
    next_seq: BTreeMap<String, u64>,
}

impl GapBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an event. Returns zero or more events that are now ready to
    /// deliver in order.
    pub fn insert(&mut self, env: Envelope) -> Vec<Envelope> {
        let stream_id = env.eel_stream_id.clone();
        let seq = env.eel_seq;
        let expected = *self.next_seq.get(&stream_id).unwrap_or(&0);

        if seq < expected {
            // Already delivered or duplicate.
            return vec![];
        }

        if seq == expected {
            let mut ready = vec![env];
            let mut cur = expected + 1;
            *self.next_seq.entry(stream_id.clone()).or_insert(0) = cur;

            let buf = self.pending.entry(stream_id.clone()).or_default();
            while let Some(buffered) = buf.remove(&cur) {
                ready.push(buffered);
                cur += 1;
            }
            *self.next_seq.entry(stream_id).or_insert(0) = cur;
            ready
        } else {
            // Future event; park until the gap is filled.
            self.pending.entry(stream_id).or_default().insert(seq, env);
            vec![]
        }
    }
}

// ---------------------------------------------------------------------------
// DLQ / tamper topic helpers
// ---------------------------------------------------------------------------

/// Returns the DLQ topic name for `topic`.
pub fn dlq_topic(topic: &str) -> String {
    format!("{}.dlq", topic)
}

/// Returns the tamper-evidence topic name for `topic`.
pub fn tamper_topic(topic: &str) -> String {
    format!("{}.tamper", topic)
}

// ---------------------------------------------------------------------------
// Deduplication store (PG-durable, Valkey write-through)
// ---------------------------------------------------------------------------

/// PG-durable deduplication store.
///
/// On each received event, call [`DedupeStore::check_and_record`]. If it
/// returns `Ok(false)`, the event was already processed and should be skipped.
pub struct DedupeStore {
    pool: sqlx::PgPool,
}

impl DedupeStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Returns `true` if the event is new and has been recorded, `false` if
    /// it was already seen (duplicate).
    pub async fn check_and_record(&self, event_id: &str) -> Result<bool, EventError> {
        let result = sqlx::query(
            r#"
            INSERT INTO eel_event_dedupe (event_id)
            VALUES ($1)
            ON CONFLICT (event_id) DO NOTHING
            RETURNING event_id
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EventError::Database(e.to_string()))?;

        Ok(result.is_some())
    }
}

// ---------------------------------------------------------------------------
// OTel span decorator
// ---------------------------------------------------------------------------

/// Returns the tracing span for a consumer handling `envelope`, with
/// correlation and causation IDs attached as span fields.
pub fn consumer_span(envelope: &Envelope) -> tracing::Span {
    info_span!(
        "eel.consumer.handle",
        otel.name = "eel.consumer.handle",
        event.id = %envelope.id,
        event.type = %envelope.event_type,
        eel.stream_id = %envelope.eel_stream_id,
        eel.seq = envelope.eel_seq,
        eel.correlation_id = ?envelope.eel_correlation_id,
        eel.causation_id = ?envelope.eel_causation_id,
    )
}

/// Drive a consumer for one event: verify signature, check dedupe (if store
/// provided), apply gap buffering, then call [`EventConsumer::handle`] inside
/// an OTel span.
///
/// Tamper-detected events are routed to `.tamper`; this function returns the
/// tamper topic name in `Err(EventError::TamperDetected)` so the caller can
/// forward the raw bytes.
pub async fn dispatch<C: EventConsumer>(
    consumer: &C,
    envelope: Envelope,
    key_provider: &dyn SpiffeKeyProvider,
    gap_buffer: &mut GapBuffer,
    dedupe: Option<&DedupeStore>,
) -> Result<(), EventError> {
    // Signature check.
    if let Err(EventError::TamperDetected) = verify_signature(&envelope, key_provider) {
        warn!(event.id = %envelope.id, "tamper detected");
        return Err(EventError::TamperDetected);
    }

    // Deduplication.
    if let Some(store) = dedupe {
        if !store.check_and_record(&envelope.id).await? {
            return Ok(()); // Already processed; idempotent skip.
        }
    }

    // In-order buffering.
    let ready = gap_buffer.insert(envelope);
    for env in ready {
        let span = consumer_span(&env);
        consumer.handle(env).instrument(span).await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Kafka consumer (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "kafka")]
pub mod kafka {
    //! Kafka stream consumer integration.
    //!
    //! Uses `rdkafka::consumer::StreamConsumer` and a transactional producer
    //! for DLQ / tamper routing. The application creates a [`KafkaConsumerRunner`]
    //! and calls [`KafkaConsumerRunner::run`] in a spawned task.

    use rdkafka::{
        config::ClientConfig,
        consumer::{Consumer, StreamConsumer},
        message::Message,
        producer::{FutureProducer, FutureRecord},
    };
    use std::time::Duration;
    use tracing::{error, info, warn};

    use super::{dlq_topic, tamper_topic, GapBuffer, SpiffeKeyProvider};
    use crate::{consumer::dispatch, envelope::Envelope, error::EventError, EventConsumer};

    /// Runs a Kafka consumer group that delivers events to an [`EventConsumer`]
    /// with signature verification, PG-durable deduplication, gap buffering,
    /// DLQ routing, and tamper routing.
    pub struct KafkaConsumerRunner {
        brokers: String,
        group_id: String,
        topics: Vec<String>,
    }

    impl KafkaConsumerRunner {
        pub fn new(
            brokers: impl Into<String>,
            group_id: impl Into<String>,
            topics: Vec<String>,
        ) -> Self {
            Self {
                brokers: brokers.into(),
                group_id: group_id.into(),
                topics,
            }
        }

        /// Run until cancelled. Errors on unrecoverable consumer setup failures.
        pub async fn run<C: EventConsumer + 'static>(
            &self,
            consumer_impl: C,
            key_provider: &dyn SpiffeKeyProvider,
            pool: Option<sqlx::PgPool>,
        ) -> Result<(), EventError> {
            let kafka_consumer: StreamConsumer = ClientConfig::new()
                .set("bootstrap.servers", &self.brokers)
                .set("group.id", &self.group_id)
                .set("enable.auto.commit", "false")
                .set("auto.offset.reset", "earliest")
                .create()
                .map_err(|e| EventError::Concurrency(e.to_string()))?;

            let topic_strs: Vec<&str> = self.topics.iter().map(String::as_str).collect();
            kafka_consumer
                .subscribe(&topic_strs)
                .map_err(|e| EventError::Concurrency(e.to_string()))?;

            let producer: FutureProducer = ClientConfig::new()
                .set("bootstrap.servers", &self.brokers)
                .create()
                .map_err(|e| EventError::Concurrency(e.to_string()))?;

            let dedupe = pool.map(super::DedupeStore::new);
            let mut gap_buffer = GapBuffer::new();

            loop {
                match kafka_consumer.recv().await {
                    Err(e) => {
                        error!("kafka recv error: {}", e);
                    }
                    Ok(msg) => {
                        let payload = match msg.payload() {
                            Some(b) => b,
                            None => continue,
                        };

                        let envelope: Envelope = match serde_json::from_slice(payload) {
                            Ok(e) => e,
                            Err(err) => {
                                warn!("envelope decode failed: {err}");
                                let topic = msg.topic();
                                let _ = producer
                                    .send(
                                        FutureRecord::to(&dlq_topic(topic))
                                            .payload(payload)
                                            .key(""),
                                        Duration::from_secs(5),
                                    )
                                    .await;
                                continue;
                            }
                        };

                        let topic_name = msg.topic().to_string();
                        match dispatch(
                            &consumer_impl,
                            envelope,
                            key_provider,
                            &mut gap_buffer,
                            dedupe.as_ref(),
                        )
                        .await
                        {
                            Ok(()) => {
                                info!("event processed");
                            }
                            Err(EventError::TamperDetected) => {
                                let _ = producer
                                    .send(
                                        FutureRecord::to(&tamper_topic(&topic_name))
                                            .payload(payload)
                                            .key(""),
                                        Duration::from_secs(5),
                                    )
                                    .await;
                            }
                            Err(e) => {
                                error!("consumer error: {e}");
                                let _ = producer
                                    .send(
                                        FutureRecord::to(&dlq_topic(&topic_name))
                                            .payload(payload)
                                            .key(""),
                                        Duration::from_secs(5),
                                    )
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    }
}
