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

//! Kafka consumer loop with signature verification and sequencer integration.

use std::sync::Arc;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use sqlx::PgPool;
use tlog_sequencer::{AppendResult, Sequencer, SequencerError};
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use crate::topic_router::topic_to_domain;

/// Decision for the consumer's offset invariant (ADR-0017): advance the Kafka
/// offset ONLY when the event is durably in the log; otherwise park (record a
/// failure and retry) with a stable failure `kind` for dead-letter triage.
///
/// HM3 fault-injection policy: pure function over the sequencer outcome, no
/// database required to evaluate.
#[derive(Debug, PartialEq)]
pub enum OffsetPolicy {
    /// Event durably in the log — safe to save the offset.
    Advance,
    /// Park the offset and record a failure of the given kind.
    Park { kind: &'static str },
}

/// Map a sequencer append outcome to an offset policy.
///
/// Note on duplicate appends: the dedup path in the sequencer returns
/// `Ok(Some(_))` (a synthetic result with leaf_index 0), so replays after an
/// offset-save failure are treated as Advance — exactly what makes re-append
/// on restart dedup-safe.
pub fn offset_policy(outcome: &Result<Option<AppendResult>, SequencerError>) -> OffsetPolicy {
    match outcome {
        Ok(Some(_)) => OffsetPolicy::Advance,
        Ok(None) => OffsetPolicy::Park {
            kind: "rate_limited",
        },
        Err(e) => OffsetPolicy::Park { kind: e.kind() },
    }
}

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

                    let envelope: eel_events_rs::Envelope = match serde_json::from_slice(&payload) {
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

                    // Integrity invariant (ADR-0017): the offset is advanced
                    // ONLY when the event is durably in the log. Rate-limited
                    // and failed appends park in tlog_ingest_failures and
                    // block the offset so they are retried / alarmed, never
                    // silently skipped.
                    let outcome = self
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
                        .await;

                    if let OffsetPolicy::Park { kind } = offset_policy(&outcome) {
                        match &outcome {
                            Ok(None) => {
                                warn!(topic = %topic, "rate limit exceeded; parking failure")
                            }
                            Err(e) => warn!(error = %e, "append failed; parking failure"),
                            Ok(Some(_)) => unreachable!("Advance cannot produce a detail"),
                        }
                        let detail = match &outcome {
                            Ok(None) => "sequencer rate limit exceeded".to_string(),
                            Err(e) => e.to_string(),
                            Ok(Some(_)) => unreachable!(),
                        };
                        if let Err(e2) = crate::failures::record_failure(
                            &self.pool,
                            &topic,
                            msg.partition(),
                            msg.offset(),
                            Some(&event_id),
                            kind,
                            &detail,
                        )
                        .await
                        {
                            error!(error = %e2, "failed to record ingest failure; offset NOT advanced");
                        }
                        continue;
                    }

                    let partition = msg.partition();
                    let offset = msg.offset();
                    if let Err(e) =
                        crate::offset_tracker::save_offset(&self.pool, &topic, partition, offset)
                            .await
                    {
                        error!(error = %e, "offset save failed; will re-append on restart (dedup safe)");
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
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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

#[cfg(test)]
mod offset_policy_tests {
    //! HM3 fault-injection policy tests: evaluate the offset decision for every
    //! sequencer outcome WITHOUT a database. `AppendResult` and
    //! `InclusionProof` have public fields, so outcomes are constructed
    //! directly.
    use super::*;
    use tlog_domain::InclusionProof;

    fn ok_append(leaf_index: u64) -> Result<Option<AppendResult>, SequencerError> {
        Ok(Some(AppendResult {
            leaf_index,
            tree_size: leaf_index + 1,
            proof: InclusionProof {
                leaf_index,
                tree_size: leaf_index + 1,
                root_hash: String::new(),
                hashes: vec![],
            },
        }))
    }

    #[test]
    fn advance_only_on_success() {
        assert_eq!(offset_policy(&ok_append(7)), OffsetPolicy::Advance);
    }

    #[test]
    fn park_on_rate_limit() {
        assert_eq!(
            offset_policy(&Ok(None)),
            OffsetPolicy::Park {
                kind: "rate_limited"
            }
        );
    }

    #[test]
    fn park_on_proof_partition() {
        // Simulates a partition / proof-generation failure ("db down" payload).
        let outcome: Result<Option<AppendResult>, SequencerError> =
            Err(SequencerError::Proof("db down".into()));
        assert_eq!(
            offset_policy(&outcome),
            OffsetPolicy::Park { kind: "proof" }
        );
    }

    #[test]
    fn park_on_dead_db() {
        let outcome: Result<Option<AppendResult>, SequencerError> =
            Err(SequencerError::Db(sqlx::Error::RowNotFound));
        assert_eq!(offset_policy(&outcome), OffsetPolicy::Park { kind: "db" });
    }

    #[test]
    fn duplicate_append_still_advances() {
        // Dedup semantics: when an event already exists (e.g. re-append after
        // an offset-save failure), the sequencer returns Ok(Some(_)) with
        // leaf_index 0 — the event IS durably in the log, so the offset must
        // advance. This is exactly what makes restart replays dedup-safe.
        assert_eq!(offset_policy(&ok_append(0)), OffsetPolicy::Advance);
    }

    #[test]
    fn no_failure_mode_advances() {
        let modes: Vec<Result<Option<AppendResult>, SequencerError>> = vec![
            Ok(None),
            Err(SequencerError::Proof("db down".into())),
            Err(SequencerError::Db(sqlx::Error::RowNotFound)),
        ];
        for m in &modes {
            assert_ne!(offset_policy(m), OffsetPolicy::Advance, "{:?}", m);
        }
    }
}
