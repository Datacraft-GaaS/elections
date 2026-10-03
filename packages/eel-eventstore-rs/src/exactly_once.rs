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

//! Exactly-once Kafka-transactional consumer helper.
//!
//! Wraps rdkafka primitives so that message processing and offset commits
//! happen atomically inside a Kafka transaction.

use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
    producer::{BaseProducer, BaseRecord, Producer},
    Message, Offset, TopicPartitionList,
};
use std::time::Duration;
use tracing::{error, info, instrument};

use crate::error::EventStoreError;

/// Configuration for the exactly-once consumer helper.
pub struct ExactlyOnceConfig {
    pub brokers: String,
    pub group_id: String,
    pub transactional_id: String,
    pub topics: Vec<String>,
}

/// Reads from Kafka and publishes derived events via a transactional producer,
/// committing source offsets atomically with the produced messages.
pub struct ExactlyOnceConsumer {
    consumer: StreamConsumer,
    producer: BaseProducer,
}

impl ExactlyOnceConsumer {
    /// Build and initialise the consumer + transactional producer.
    pub fn new(cfg: &ExactlyOnceConfig) -> Result<Self, EventStoreError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &cfg.brokers)
            .set("group.id", &cfg.group_id)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(EventStoreError::Kafka)?;

        let topic_refs: Vec<&str> = cfg.topics.iter().map(String::as_str).collect();
        consumer
            .subscribe(&topic_refs)
            .map_err(EventStoreError::Kafka)?;

        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", &cfg.brokers)
            .set("transactional.id", &cfg.transactional_id)
            .set("enable.idempotence", "true")
            .set("acks", "all")
            .create()
            .map_err(EventStoreError::Kafka)?;

        producer
            .init_transactions(Duration::from_secs(10))
            .map_err(EventStoreError::Kafka)?;

        Ok(Self { consumer, producer })
    }

    /// Consume one message and call `handler` inside a Kafka transaction.
    ///
    /// If `handler` returns `Some((topic, payload))`, that payload is produced
    /// to `topic` before the transaction commits. The source offset is always
    /// committed inside the transaction.
    #[instrument(skip(self, handler))]
    pub async fn process_one<F, Fut>(&self, handler: F) -> Result<(), EventStoreError>
    where
        F: FnOnce(Vec<u8>) -> Fut,
        Fut: std::future::Future<Output = Result<Option<(String, Vec<u8>)>, EventStoreError>>,
    {
        let msg = self.consumer.recv().await.map_err(EventStoreError::Kafka)?;

        let payload = msg.payload().unwrap_or_default().to_vec();
        let topic = msg.topic().to_owned();
        let partition = msg.partition();
        let offset = msg.offset();

        self.producer
            .begin_transaction()
            .map_err(EventStoreError::Kafka)?;

        match handler(payload).await {
            Ok(Some((out_topic, out_payload))) => {
                self.producer
                    .send(BaseRecord::to(&out_topic).payload(&out_payload).key(""))
                    .map_err(|(e, _)| EventStoreError::Kafka(e))?;
            }
            Ok(None) => {}
            Err(e) => {
                error!(error = %e, "handler failed, aborting transaction");
                self.producer
                    .abort_transaction(Duration::from_secs(5))
                    .map_err(EventStoreError::Kafka)?;
                return Err(e);
            }
        }

        let mut offsets = TopicPartitionList::new();
        offsets
            .add_partition_offset(&topic, partition, Offset::Offset(offset + 1))
            .map_err(EventStoreError::Kafka)?;

        let group_meta = self.consumer.group_metadata().ok_or_else(|| {
            EventStoreError::Kafka(rdkafka::error::KafkaError::ConsumerCommit(
                rdkafka::types::RDKafkaErrorCode::UnknownMemberId,
            ))
        })?;

        self.producer
            .send_offsets_to_transaction(&offsets, &group_meta, Duration::from_secs(5))
            .map_err(EventStoreError::Kafka)?;

        self.producer
            .commit_transaction(Duration::from_secs(10))
            .map_err(EventStoreError::Kafka)?;

        info!(topic, partition, offset, "committed offset in transaction");
        Ok(())
    }
}
