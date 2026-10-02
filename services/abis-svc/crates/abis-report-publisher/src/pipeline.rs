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

//! Orchestrator loop: fetch -> resolve -> dispatch -> record outcome -> emit events.

use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

use eel_resilience_rs::{
	bulkhead::Bulkhead,
	circuit_breaker::CircuitBreaker,
};

use crate::{
	channels,
	domain::{Channel, DeliveryContext, DeliveryOutcome},
	metrics,
	repository,
};

const BATCH_SIZE: i64 = 32;
const POLL_INTERVAL: Duration = Duration::from_secs(5);
const CB_THRESHOLD: u32 = 5;
const CB_WINDOW: Duration = Duration::from_secs(60);
const CB_COOLDOWN: Duration = Duration::from_secs(30);

/// Run the publisher pipeline until cancelled.
pub async fn run(pool: Arc<PgPool>) {
	let breaker = CircuitBreaker::new(
		"report-publisher",
		CB_THRESHOLD,
		CB_WINDOW,
		CB_COOLDOWN,
	);
	let bulkhead = Bulkhead::new("report-publisher", 16);

	loop {
		let pool = pool.clone();

		match breaker.allow().await {
			Err(e) => {
				warn!(error = %e, "circuit breaker open; pausing publisher loop");
				tokio::time::sleep(CB_COOLDOWN).await;
				continue;
			}
			Ok(()) => {}
		}

		let deliveries = match repository::fetch_pending(&pool, BATCH_SIZE).await {
			Ok(d) => d,
			Err(e) => {
				error!(error = %e, "failed to fetch pending deliveries");
				breaker.record_failure().await;
				tokio::time::sleep(POLL_INTERVAL).await;
				continue;
			}
		};

		if deliveries.is_empty() {
			tokio::time::sleep(POLL_INTERVAL).await;
			continue;
		}

		breaker.record_success().await;

		for delivery in deliveries {
			let pool = pool.clone();
			let bulkhead = bulkhead.clone();

			tokio::spawn(async move {
				let _permit = match bulkhead.acquire().await {
					Ok(p) => p,
					Err(e) => {
						warn!(error = %e, "bulkhead full; skipping delivery");
						return;
					}
				};

				process_one(&pool, delivery).await;
			});
		}

		// Reconcile api_pull deliveries.
		reconcile_api_pull(&pool).await;

		tokio::time::sleep(POLL_INTERVAL).await;
	}
}

async fn process_one(pool: &PgPool, delivery: crate::domain::Delivery) {
	let delivery_id = delivery.delivery_id;

	// CAS: only proceed if we successfully transition to sending.
	let rows = match repository::mark_sending(pool, delivery_id).await {
		Ok(n) => n,
		Err(e) => {
			error!(%delivery_id, error = %e, "mark_sending failed");
			return;
		}
	};
	if rows == 0 {
		// Another worker grabbed it.
		return;
	}

	// Resolve context.
	let subscriber = match repository::fetch_subscriber(pool, delivery.subscription_id).await {
		Ok(Some(s)) => s,
		Ok(None) => {
			let _ = repository::abandon(pool, delivery_id, "subscriber not found").await;
			return;
		}
		Err(e) => {
			error!(%delivery_id, error = %e, "fetch_subscriber failed");
			let _ = repository::mark_failed_with_backoff(pool, delivery_id, &e.to_string()).await;
			return;
		}
	};

	// Skip suspended subscribers.
	if subscriber.suspended_at.is_some() {
		let _ = repository::abandon(pool, delivery_id, "subscriber suspended").await;
		return;
	}

	let subscription = match repository::fetch_subscription(pool, delivery.subscription_id).await {
		Ok(Some(s)) => s,
		Ok(None) => {
			let _ = repository::abandon(pool, delivery_id, "subscription not found").await;
			return;
		}
		Err(e) => {
			error!(%delivery_id, error = %e, "fetch_subscription failed");
			let _ = repository::mark_failed_with_backoff(pool, delivery_id, &e.to_string()).await;
			return;
		}
	};

	let artifact = match repository::fetch_artifact(pool, delivery.artifact_id).await {
		Ok(Some(a)) => a,
		Ok(None) => {
			let _ = repository::abandon(pool, delivery_id, "artifact not found").await;
			return;
		}
		Err(e) => {
			error!(%delivery_id, error = %e, "fetch_artifact failed");
			let _ = repository::mark_failed_with_backoff(pool, delivery_id, &e.to_string()).await;
			return;
		}
	};

	let tlog_anchor = repository::fetch_tlog_anchor(pool, delivery.artifact_id)
		.await
		.unwrap_or(None);

	let channel: Channel = match subscriber.channel.parse_channel() {
		Some(c) => c,
		None => {
			let _ = repository::abandon(
				pool,
				delivery_id,
				&format!("unknown channel: {}", subscriber.channel),
			)
			.await;
			return;
		}
	};

	let ctx = DeliveryContext {
		delivery: delivery.clone(),
		subscriber,
		subscription,
		artifact,
		tlog_anchor,
	};

	let start = std::time::Instant::now();
	let outcome = channels::dispatch(&channel, ctx).await;
	let elapsed_ms = start.elapsed().as_millis() as f64;

	metrics::delivery_latency_ms()
		.with_label_values(&[&channel.to_string()])
		.observe(elapsed_ms);

	match outcome {
		DeliveryOutcome::Succeeded {
			remote_receipt_id,
			remote_receipt_at,
		} => {
			let _ = repository::mark_delivered(pool, delivery_id, remote_receipt_id, remote_receipt_at).await;
			metrics::deliveries_total()
				.with_label_values(&[&channel.to_string(), "delivered"])
				.inc();
			info!(%delivery_id, "delivery succeeded");
			emit_event("eel.abis.report_delivery_succeeded.v1", delivery_id).await;
		}
		DeliveryOutcome::Retryable { error } => {
			// api_pull hold: keep status pending without incrementing attempt_count.
			if error.starts_with("api_pull:") {
				// Reset to pending without backoff.
				let _ = sqlx::query(
					"UPDATE eel_report_deliveries SET status = 'pending' WHERE delivery_id = $1",
				)
				.bind(delivery_id)
				.execute(pool)
				.await;
				return;
			}

			let _ = repository::mark_failed_with_backoff(pool, delivery_id, &error).await;
			metrics::deliveries_total()
				.with_label_values(&[&channel.to_string(), "failed"])
				.inc();
			warn!(%delivery_id, %error, "delivery failed; will retry");
			emit_event("eel.abis.report_delivery_attempted.v1", delivery_id).await;

			// Check if now abandoned (mark_failed_with_backoff sets abandoned when >= max).
			if let Ok(Some(d)) = repository::fetch_delivery(pool, delivery_id).await {
				if d.status == "abandoned" {
					metrics::abandon_total()
						.with_label_values(&[&channel.to_string()])
						.inc();
					emit_event("eel.abis.report_delivery_abandoned.v1", delivery_id).await;
				}
			}
		}
		DeliveryOutcome::Permanent { error } => {
			let _ = repository::abandon(pool, delivery_id, &error).await;
			metrics::abandon_total()
				.with_label_values(&[&channel.to_string()])
				.inc();
			error!(%delivery_id, %error, "delivery permanently failed; abandoning");
			emit_event("eel.abis.report_delivery_abandoned.v1", delivery_id).await;
		}
	}
}

/// Reconcile api_pull deliveries: mark delivered when access log has a row.
async fn reconcile_api_pull(pool: &PgPool) {
	let pending: Vec<crate::domain::Delivery> = sqlx::query_as::<_, crate::domain::Delivery>(
		r#"
		SELECT
			d.delivery_id, d.run_id, d.subscription_id, d.artifact_id,
			d.status, d.attempt_count, d.max_attempts,
			d.next_attempt_at, d.last_attempt_at, d.delivered_at,
			d.last_error, d.remote_receipt_id, d.remote_receipt_at
		FROM eel_report_deliveries d
		JOIN eel_report_subscriptions sub ON sub.subscription_id = d.subscription_id
		JOIN eel_report_subscribers s ON s.subscriber_id = sub.subscriber_id
		WHERE d.status = 'pending'
		  AND s.channel = 'api_pull'
		LIMIT 64
		"#,
	)
	.fetch_all(pool)
	.await
	.unwrap_or_default();

	for d in pending {
		if let Ok(true) = repository::api_pull_observed(pool, d.artifact_id).await {
			let _ = repository::mark_delivered(pool, d.delivery_id, None, None).await;
			info!(delivery_id = %d.delivery_id, "api_pull delivery marked delivered");
		}
	}
}

async fn emit_event(event_type: &str, delivery_id: Uuid) {
	// In production: use eel-events-rs to publish to the event bus.
	// Stubbed here to avoid requiring a Kafka broker at test time.
	tracing::debug!(event_type, %delivery_id, "emit event (stub)");
}

/// Extension trait to parse channel string without bringing in domain variants.
trait ParseChannel {
	fn parse_channel(&self) -> Option<Channel>;
}

impl ParseChannel for str {
	fn parse_channel(&self) -> Option<Channel> {
		match self {
			"email" => Some(Channel::Email),
			"webhook" => Some(Channel::Webhook),
			"kafka" => Some(Channel::Kafka),
			"sftp" => Some(Channel::Sftp),
			"gov_printer" => Some(Channel::GovPrinter),
			"kbc_sftp" => Some(Channel::KbcSftp),
			"api_pull" => Some(Channel::ApiPull),
			_ => None,
		}
	}
}
