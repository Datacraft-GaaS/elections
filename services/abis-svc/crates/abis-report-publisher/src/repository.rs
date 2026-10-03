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

//! Database queries for the delivery lifecycle.
//! Uses runtime sqlx (non-macro) queries to avoid requiring DATABASE_URL at compile time.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Artifact, Delivery, Subscriber, Subscription, TlogAnchor};

/// Fetch up to `limit` deliveries eligible for dispatch.
/// Eligible: (pending OR failed) AND (next_attempt_at IS NULL OR next_attempt_at <= NOW()).
/// Uses FOR UPDATE SKIP LOCKED so concurrent workers do not race.
pub async fn fetch_pending(pool: &PgPool, limit: i64) -> sqlx::Result<Vec<Delivery>> {
    sqlx::query_as::<_, Delivery>(
        r#"
		SELECT
			delivery_id, run_id, subscription_id, artifact_id,
			status, attempt_count, max_attempts,
			next_attempt_at, last_attempt_at, delivered_at,
			last_error, remote_receipt_id, remote_receipt_at
		FROM eel_report_deliveries
		WHERE status IN ('pending', 'failed')
		  AND (next_attempt_at IS NULL OR next_attempt_at <= NOW())
		ORDER BY next_attempt_at ASC NULLS FIRST, delivery_id
		LIMIT $1
		FOR UPDATE SKIP LOCKED
		"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// CAS: transition delivery from pending|failed -> sending.
/// Returns 0 if the row was not in the expected state (idempotency guard).
pub async fn mark_sending(pool: &PgPool, delivery_id: Uuid) -> sqlx::Result<u64> {
    let rows = sqlx::query(
        r#"
		UPDATE eel_report_deliveries
		SET status = 'sending', last_attempt_at = NOW()
		WHERE delivery_id = $1
		  AND status IN ('pending', 'failed')
		"#,
    )
    .bind(delivery_id)
    .execute(pool)
    .await?;
    Ok(rows.rows_affected())
}

/// Mark delivery as delivered. Idempotent: if already delivered, does not change delivered_at.
pub async fn mark_delivered(
    pool: &PgPool,
    delivery_id: Uuid,
    remote_receipt_id: Option<String>,
    remote_receipt_at: Option<DateTime<Utc>>,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
		UPDATE eel_report_deliveries
		SET
			status = 'delivered',
			delivered_at = COALESCE(delivered_at, NOW()),
			remote_receipt_id = COALESCE($2, remote_receipt_id),
			remote_receipt_at = COALESCE($3, remote_receipt_at)
		WHERE delivery_id = $1
		  AND status != 'abandoned'
		"#,
    )
    .bind(delivery_id)
    .bind(remote_receipt_id)
    .bind(remote_receipt_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Increment attempt_count, record error, compute next_attempt_at with exponential backoff.
/// Base: 30s * 2^attempt_count, cap 4h, +20% uniform jitter.
pub async fn mark_failed_with_backoff(
    pool: &PgPool,
    delivery_id: Uuid,
    error: &str,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
		UPDATE eel_report_deliveries
		SET
			status = CASE
				WHEN attempt_count + 1 >= max_attempts THEN 'abandoned'
				ELSE 'failed'
			END,
			attempt_count = attempt_count + 1,
			last_error    = $2,
			next_attempt_at = CASE
				WHEN attempt_count + 1 >= max_attempts THEN NULL
				ELSE NOW() + (
					LEAST(30.0 * power(2.0, attempt_count::float), 14400.0)
					* (1.0 + random() * 0.2)
				) * INTERVAL '1 second'
			END
		WHERE delivery_id = $1
		  AND status IN ('sending', 'failed', 'pending')
		"#,
    )
    .bind(delivery_id)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

/// Explicitly abandon a delivery (admin action or circuit-breaker decision).
pub async fn abandon(pool: &PgPool, delivery_id: Uuid, reason: &str) -> sqlx::Result<()> {
    sqlx::query(
        r#"
		UPDATE eel_report_deliveries
		SET status = 'abandoned', last_error = $2
		WHERE delivery_id = $1
		  AND status != 'delivered'
		"#,
    )
    .bind(delivery_id)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

/// Reset a delivery to pending so the pipeline picks it up on the next tick.
pub async fn retry(pool: &PgPool, delivery_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        r#"
		UPDATE eel_report_deliveries
		SET status = 'pending', next_attempt_at = NOW(), last_error = NULL
		WHERE delivery_id = $1
		  AND status IN ('failed', 'abandoned')
		"#,
    )
    .bind(delivery_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Cancel a delivery (idempotent; delivered rows are not touched).
pub async fn cancel(pool: &PgPool, delivery_id: Uuid) -> sqlx::Result<()> {
    sqlx::query(
        r#"
		UPDATE eel_report_deliveries
		SET status = 'abandoned', last_error = 'cancelled by operator'
		WHERE delivery_id = $1
		  AND status NOT IN ('delivered', 'abandoned')
		"#,
    )
    .bind(delivery_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Fetch one delivery by ID (for admin API).
pub async fn fetch_delivery(pool: &PgPool, delivery_id: Uuid) -> sqlx::Result<Option<Delivery>> {
    sqlx::query_as::<_, Delivery>(
        r#"
		SELECT
			delivery_id, run_id, subscription_id, artifact_id,
			status, attempt_count, max_attempts,
			next_attempt_at, last_attempt_at, delivered_at,
			last_error, remote_receipt_id, remote_receipt_at
		FROM eel_report_deliveries
		WHERE delivery_id = $1
		"#,
    )
    .bind(delivery_id)
    .fetch_optional(pool)
    .await
}

/// List deliveries for a given run_id, paginated.
pub async fn list_by_run(
    pool: &PgPool,
    run_id: Uuid,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<Delivery>> {
    sqlx::query_as::<_, Delivery>(
        r#"
		SELECT
			delivery_id, run_id, subscription_id, artifact_id,
			status, attempt_count, max_attempts,
			next_attempt_at, last_attempt_at, delivered_at,
			last_error, remote_receipt_id, remote_receipt_at
		FROM eel_report_deliveries
		WHERE run_id = $1
		ORDER BY delivery_id
		LIMIT $2 OFFSET $3
		"#,
    )
    .bind(run_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Fetch subscriber for a subscription.
pub async fn fetch_subscriber(
    pool: &PgPool,
    subscription_id: Uuid,
) -> sqlx::Result<Option<Subscriber>> {
    sqlx::query_as::<_, Subscriber>(
        r#"
		SELECT
			s.subscriber_id, s.name, s.channel, s.endpoint, s.endpoint_hash,
			s.audience, s.auth_ref, s.pgp_pubkey_ref, s.preferred_format,
			s.created_at, s.verified_at, s.suspended_at, s.suspension_reason
		FROM eel_report_subscribers s
		JOIN eel_report_subscriptions sub ON sub.subscriber_id = s.subscriber_id
		WHERE sub.subscription_id = $1
		"#,
    )
    .bind(subscription_id)
    .fetch_optional(pool)
    .await
}

/// Fetch subscription.
pub async fn fetch_subscription(
    pool: &PgPool,
    subscription_id: Uuid,
) -> sqlx::Result<Option<Subscription>> {
    sqlx::query_as::<_, Subscription>(
        r#"
		SELECT subscription_id, subscriber_id, definition_id, format, created_at, cancelled_at
		FROM eel_report_subscriptions
		WHERE subscription_id = $1
		"#,
    )
    .bind(subscription_id)
    .fetch_optional(pool)
    .await
}

/// Fetch artifact.
pub async fn fetch_artifact(pool: &PgPool, artifact_id: Uuid) -> sqlx::Result<Option<Artifact>> {
    sqlx::query_as::<_, Artifact>(
        r#"
		SELECT
			artifact_id, run_id, format, blob_ref, sha256, size_bytes,
			hsm_signature, rendered_at
		FROM eel_report_artifacts
		WHERE artifact_id = $1
		"#,
    )
    .bind(artifact_id)
    .fetch_optional(pool)
    .await
}

/// Fetch tlog anchor for an artifact if present.
pub async fn fetch_tlog_anchor(
    pool: &PgPool,
    artifact_id: Uuid,
) -> sqlx::Result<Option<TlogAnchor>> {
    sqlx::query_as::<_, TlogAnchor>(
        r#"
		SELECT anchor_id, run_id, artifact_id, tlog_leaf_index
		FROM eel_report_tlog_anchors
		WHERE artifact_id = $1
		"#,
    )
    .bind(artifact_id)
    .fetch_optional(pool)
    .await
}

/// Check whether an api_pull delivery has been observed in the access log.
pub async fn api_pull_observed(pool: &PgPool, artifact_id: Uuid) -> sqlx::Result<bool> {
    let row: (i64,) =
        sqlx::query_as(r#"SELECT COUNT(*) FROM eel_report_access_log WHERE artifact_id = $1"#)
            .bind(artifact_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0 > 0)
}
