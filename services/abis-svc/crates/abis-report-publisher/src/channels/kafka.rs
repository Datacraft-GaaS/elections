//! Kafka channel adapter.
//!
//! Produces to the topic stored in the subscriber's endpoint field
//! (default: eel.reports.published.v1 if endpoint is empty or "-").
//! Payload: JSON with run_id, artifact_id, format, sha256, minio_ref, tlog_leaf_index.

use chrono::Utc;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use tracing::{info, warn};

use crate::domain::{DeliveryContext, DeliveryOutcome};

const DEFAULT_TOPIC: &str = "eel.reports.published.v1";

pub async fn deliver(ctx: DeliveryContext) -> DeliveryOutcome {
	let brokers = std::env::var("EEL_KAFKA_BROKERS")
		.unwrap_or_else(|_| "localhost:9092".into());

	let topic = {
		let ep = ctx.subscriber.endpoint.trim();
		if ep.is_empty() || ep == "-" {
			DEFAULT_TOPIC.to_string()
		} else {
			ep.to_string()
		}
	};

	let producer: FutureProducer = match ClientConfig::new()
		.set("bootstrap.servers", &brokers)
		.set("message.timeout.ms", "10000")
		.create()
	{
		Ok(p) => p,
		Err(e) => {
			return DeliveryOutcome::Retryable {
				error: format!("kafka producer create failed: {e}"),
			}
		}
	};

	let payload = serde_json::json!({
		"run_id": ctx.delivery.run_id,
		"artifact_id": ctx.artifact.artifact_id,
		"format": ctx.artifact.format,
		"sha256": hex::encode(&ctx.artifact.sha256),
		"minio_ref": ctx.artifact.blob_ref,
		"tlog_leaf_index": ctx.tlog_anchor.as_ref().map(|a| a.tlog_leaf_index),
	});

	let key = ctx.delivery.delivery_id.to_string();
	let body = match serde_json::to_string(&payload) {
		Ok(s) => s,
		Err(e) => {
			return DeliveryOutcome::Permanent {
				error: format!("payload serialization failed: {e}"),
			}
		}
	};

	let record = FutureRecord::to(&topic)
		.key(key.as_str())
		.payload(body.as_str());

	match producer.send(record, Duration::from_secs(10)).await {
		Ok((partition, offset)) => {
			info!(
				delivery_id = %ctx.delivery.delivery_id,
				topic = %topic,
				partition,
				offset,
				"kafka message produced"
			);
			DeliveryOutcome::Succeeded {
				remote_receipt_id: Some(format!("{topic}/{partition}/{offset}")),
				remote_receipt_at: Some(Utc::now()),
			}
		}
		Err((e, _)) => {
			warn!(delivery_id = %ctx.delivery.delivery_id, error = %e, "kafka produce failed");
			DeliveryOutcome::Retryable {
				error: format!("kafka produce failed: {e}"),
			}
		}
	}
}
