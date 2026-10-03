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

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    ReportAccessLogEntry, ReportArtifact, ReportDefinition, ReportDelivery, ReportRun,
    ReportTlogAnchor,
};
use crate::error::Result;

// ---------------------------------------------------------------------------
// Definitions
// ---------------------------------------------------------------------------

pub async fn fetch_active_definitions(
    pool: &PgPool,
    class: Option<&str>,
) -> Result<Vec<ReportDefinition>> {
    let rows = if let Some(cls) = class {
        sqlx::query_as::<_, ReportDefinition>(
            "SELECT definition_id, slug, title, report_class, audience, data_class, \
			 sql_template_ref, renderer_pipeline, requires_hsm_sign, requires_tlog_anchor, \
			 default_schedule, timezone, retention_years, template_version, \
			 cosign_signature, created_at, updated_at, retired_at, retirement_reason \
			 FROM eel_report_definitions \
			 WHERE retired_at IS NULL AND report_class = $1 \
			 ORDER BY report_class, slug",
        )
        .bind(cls)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, ReportDefinition>(
            "SELECT definition_id, slug, title, report_class, audience, data_class, \
			 sql_template_ref, renderer_pipeline, requires_hsm_sign, requires_tlog_anchor, \
			 default_schedule, timezone, retention_years, template_version, \
			 cosign_signature, created_at, updated_at, retired_at, retirement_reason \
			 FROM eel_report_definitions \
			 WHERE retired_at IS NULL \
			 ORDER BY report_class, slug",
        )
        .fetch_all(pool)
        .await?
    };
    Ok(rows)
}

pub async fn fetch_definition_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<ReportDefinition>> {
    let row = sqlx::query_as::<_, ReportDefinition>(
        "SELECT definition_id, slug, title, report_class, audience, data_class, \
		 sql_template_ref, renderer_pipeline, requires_hsm_sign, requires_tlog_anchor, \
		 default_schedule, timezone, retention_years, template_version, \
		 cosign_signature, created_at, updated_at, retired_at, retirement_reason \
		 FROM eel_report_definitions \
		 WHERE slug = $1 AND retired_at IS NULL",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn insert_definition(pool: &PgPool, def: &ReportDefinition) -> Result<()> {
    sqlx::query(
        "INSERT INTO eel_report_definitions \
		 (definition_id, slug, title, report_class, audience, data_class, \
		  sql_template_ref, renderer_pipeline, requires_hsm_sign, requires_tlog_anchor, \
		  default_schedule, timezone, retention_years, template_version) \
		 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
    )
    .bind(def.definition_id)
    .bind(&def.slug)
    .bind(&def.title)
    .bind(&def.report_class)
    .bind(&def.audience)
    .bind(&def.data_class)
    .bind(def.sql_template_ref.as_deref())
    .bind(&def.renderer_pipeline)
    .bind(def.requires_hsm_sign)
    .bind(def.requires_tlog_anchor)
    .bind(def.default_schedule.as_deref())
    .bind(&def.timezone)
    .bind(def.retention_years)
    .bind(&def.template_version)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Runs
// ---------------------------------------------------------------------------

pub async fn create_run(
    pool: &PgPool,
    definition_id: Uuid,
    slug: &str,
    window_from: DateTime<Utc>,
    window_to: DateTime<Utc>,
    requested_by: &str,
    correlation_id: Option<Uuid>,
    trace_id: Option<&[u8]>,
) -> Result<ReportRun> {
    let run_id = Uuid::now_v7();
    let row = sqlx::query_as::<_, ReportRun>(
        "INSERT INTO eel_report_runs \
		 (run_id, definition_id, slug, window_from, window_to, requested_by, correlation_id, trace_id) \
		 VALUES ($1,$2,$3,$4,$5,$6,$7,$8) \
		 ON CONFLICT (definition_id, window_from, window_to, requested_by) \
		 DO UPDATE SET run_id = eel_report_runs.run_id \
		 RETURNING run_id, definition_id, slug, window_from, window_to, \
		   status, requested_by, requested_at, started_at, completed_at, \
		   rows_extracted, extraction_ms, render_ms, total_ms, \
		   failure_reason, correlation_id, trace_id",
    )
    .bind(run_id)
    .bind(definition_id)
    .bind(slug)
    .bind(window_from)
    .bind(window_to)
    .bind(requested_by)
    .bind(correlation_id)
    .bind(trace_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn fetch_run(pool: &PgPool, run_id: Uuid) -> Result<Option<ReportRun>> {
    let row = sqlx::query_as::<_, ReportRun>(
        "SELECT run_id, definition_id, slug, window_from, window_to, \
		   status, requested_by, requested_at, started_at, completed_at, \
		   rows_extracted, extraction_ms, render_ms, total_ms, \
		   failure_reason, correlation_id, trace_id \
		 FROM eel_report_runs WHERE run_id = $1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Compare-and-swap status transition. Returns true if the row was updated.
pub async fn advance_run_status(
    pool: &PgPool,
    run_id: Uuid,
    from_status: &str,
    to_status: &str,
) -> Result<bool> {
    let result = sqlx::query(
		"UPDATE eel_report_runs SET status = $1, \
		   started_at   = CASE WHEN $1 = 'extracting' THEN NOW() ELSE started_at END, \
		   completed_at = CASE WHEN $1 IN ('complete','failed','cancelled') THEN NOW() ELSE completed_at END \
		 WHERE run_id = $2 AND status = $3",
	)
	.bind(to_status)
	.bind(run_id)
	.bind(from_status)
	.execute(pool)
	.await?;
    Ok(result.rows_affected() == 1)
}

pub async fn fail_run(pool: &PgPool, run_id: Uuid, reason: &str) -> Result<()> {
    sqlx::query(
        "UPDATE eel_report_runs \
		 SET status = 'failed', failure_reason = $1, completed_at = NOW() \
		 WHERE run_id = $2",
    )
    .bind(reason)
    .bind(run_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_run_extraction_stats(
    pool: &PgPool,
    run_id: Uuid,
    rows_extracted: i64,
    extraction_ms: i32,
) -> Result<()> {
    sqlx::query(
        "UPDATE eel_report_runs SET rows_extracted = $1, extraction_ms = $2 WHERE run_id = $3",
    )
    .bind(rows_extracted)
    .bind(extraction_ms)
    .bind(run_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_queued_runs(pool: &PgPool, limit: i64) -> Result<Vec<ReportRun>> {
    let rows = sqlx::query_as::<_, ReportRun>(
        "SELECT run_id, definition_id, slug, window_from, window_to, \
		   status, requested_by, requested_at, started_at, completed_at, \
		   rows_extracted, extraction_ms, render_ms, total_ms, \
		   failure_reason, correlation_id, trace_id \
		 FROM eel_report_runs \
		 WHERE status = 'queued' \
		 ORDER BY requested_at \
		 LIMIT $1 FOR UPDATE SKIP LOCKED",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Artifacts
// ---------------------------------------------------------------------------

pub async fn insert_artifact(
    pool: &PgPool,
    artifact_id: Uuid,
    run_id: Uuid,
    format: &str,
    blob_ref: &str,
    sha256: &[u8],
    size_bytes: i64,
    page_count: Option<i32>,
    row_count: Option<i64>,
) -> Result<ReportArtifact> {
    let row = sqlx::query_as::<_, ReportArtifact>(
        "INSERT INTO eel_report_artifacts \
		 (artifact_id, run_id, format, blob_ref, sha256, size_bytes, page_count, row_count) \
		 VALUES ($1,$2,$3,$4,$5,$6,$7,$8) \
		 RETURNING artifact_id, run_id, format, blob_ref, sha256, size_bytes, \
		   page_count, row_count, hsm_signature, hsm_key_role, rendered_at",
    )
    .bind(artifact_id)
    .bind(run_id)
    .bind(format)
    .bind(blob_ref)
    .bind(sha256)
    .bind(size_bytes)
    .bind(page_count)
    .bind(row_count)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn fetch_artifact_by_run_format(
    pool: &PgPool,
    run_id: Uuid,
    format: &str,
) -> Result<Option<ReportArtifact>> {
    let row = sqlx::query_as::<_, ReportArtifact>(
        "SELECT artifact_id, run_id, format, blob_ref, sha256, size_bytes, \
		   page_count, row_count, hsm_signature, hsm_key_role, rendered_at \
		 FROM eel_report_artifacts WHERE run_id = $1 AND format = $2",
    )
    .bind(run_id)
    .bind(format)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn fetch_artifacts_by_run(pool: &PgPool, run_id: Uuid) -> Result<Vec<ReportArtifact>> {
    let rows = sqlx::query_as::<_, ReportArtifact>(
        "SELECT artifact_id, run_id, format, blob_ref, sha256, size_bytes, \
		   page_count, row_count, hsm_signature, hsm_key_role, rendered_at \
		 FROM eel_report_artifacts WHERE run_id = $1 ORDER BY format",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn fetch_artifact(pool: &PgPool, artifact_id: Uuid) -> Result<Option<ReportArtifact>> {
    let row = sqlx::query_as::<_, ReportArtifact>(
        "SELECT artifact_id, run_id, format, blob_ref, sha256, size_bytes, \
		   page_count, row_count, hsm_signature, hsm_key_role, rendered_at \
		 FROM eel_report_artifacts WHERE artifact_id = $1",
    )
    .bind(artifact_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn update_artifact_signature(
    pool: &PgPool,
    artifact_id: Uuid,
    hsm_signature: &[u8],
    hsm_key_role: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE eel_report_artifacts SET hsm_signature = $1, hsm_key_role = $2 \
		 WHERE artifact_id = $3",
    )
    .bind(hsm_signature)
    .bind(hsm_key_role)
    .bind(artifact_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tlog anchors
// ---------------------------------------------------------------------------

pub async fn insert_tlog_anchor(
    pool: &PgPool,
    anchor_id: Uuid,
    run_id: Uuid,
    artifact_id: Uuid,
    tlog_leaf_index: i64,
    tlog_root_hash: &[u8],
    inclusion_proof: &[u8],
    sth_size: i64,
    witness_cosigs: serde_json::Value,
) -> Result<ReportTlogAnchor> {
    let row = sqlx::query_as::<_, ReportTlogAnchor>(
        "INSERT INTO eel_report_tlog_anchors \
		 (anchor_id, run_id, artifact_id, tlog_leaf_index, tlog_root_hash, \
		  inclusion_proof, sth_size, witness_cosigs) \
		 VALUES ($1,$2,$3,$4,$5,$6,$7,$8) \
		 RETURNING anchor_id, run_id, artifact_id, tlog_leaf_index, tlog_root_hash, \
		   inclusion_proof, sth_size, witness_cosigs, anchored_at",
    )
    .bind(anchor_id)
    .bind(run_id)
    .bind(artifact_id)
    .bind(tlog_leaf_index)
    .bind(tlog_root_hash)
    .bind(inclusion_proof)
    .bind(sth_size)
    .bind(witness_cosigs)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn fetch_tlog_anchor_by_artifact(
    pool: &PgPool,
    artifact_id: Uuid,
) -> Result<Option<ReportTlogAnchor>> {
    let row = sqlx::query_as::<_, ReportTlogAnchor>(
        "SELECT anchor_id, run_id, artifact_id, tlog_leaf_index, tlog_root_hash, \
		   inclusion_proof, sth_size, witness_cosigs, anchored_at \
		 FROM eel_report_tlog_anchors WHERE artifact_id = $1",
    )
    .bind(artifact_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

// ---------------------------------------------------------------------------
// Deliveries
// ---------------------------------------------------------------------------

pub async fn fetch_pending_deliveries(pool: &PgPool, limit: i64) -> Result<Vec<ReportDelivery>> {
    let rows = sqlx::query_as::<_, ReportDelivery>(
        "SELECT delivery_id, run_id, subscription_id, artifact_id, status, \
		   attempt_count, max_attempts, next_attempt_at, last_attempt_at, \
		   delivered_at, last_error, remote_receipt_id, remote_receipt_at \
		 FROM eel_report_deliveries \
		 WHERE status IN ('pending','failed') \
		   AND (next_attempt_at IS NULL OR next_attempt_at <= NOW()) \
		   AND attempt_count < max_attempts \
		 ORDER BY next_attempt_at NULLS FIRST \
		 LIMIT $1 FOR UPDATE SKIP LOCKED",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn insert_delivery(
    pool: &PgPool,
    delivery_id: Uuid,
    run_id: Uuid,
    subscription_id: Uuid,
    artifact_id: Uuid,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO eel_report_deliveries (delivery_id, run_id, subscription_id, artifact_id) \
		 VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING",
    )
    .bind(delivery_id)
    .bind(run_id)
    .bind(subscription_id)
    .bind(artifact_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_delivery(pool: &PgPool, delivery_id: Uuid) -> Result<Option<ReportDelivery>> {
    let row = sqlx::query_as::<_, ReportDelivery>(
        "SELECT delivery_id, run_id, subscription_id, artifact_id, status, \
		   attempt_count, max_attempts, next_attempt_at, last_attempt_at, \
		   delivered_at, last_error, remote_receipt_id, remote_receipt_at \
		 FROM eel_report_deliveries WHERE delivery_id = $1",
    )
    .bind(delivery_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

// ---------------------------------------------------------------------------
// Access log
// ---------------------------------------------------------------------------

pub async fn record_access(
    pool: &PgPool,
    access_id: Uuid,
    artifact_id: Uuid,
    subject_id: Option<&str>,
    ip_prefix: Option<&str>,
    ua_hash: Option<&[u8]>,
    byte_range: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO eel_report_access_log \
		 (access_id, artifact_id, subject_id, ip_prefix, user_agent_hash, byte_range) \
		 VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(access_id)
    .bind(artifact_id)
    .bind(subject_id)
    .bind(ip_prefix)
    .bind(ua_hash)
    .bind(byte_range)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_access_log(
    pool: &PgPool,
    artifact_id: Uuid,
    limit: i64,
) -> Result<Vec<ReportAccessLogEntry>> {
    let rows = sqlx::query_as::<_, ReportAccessLogEntry>(
		"SELECT access_id, artifact_id, subject_id, ip_prefix, user_agent_hash, accessed_at, byte_range \
		 FROM eel_report_access_log \
		 WHERE artifact_id = $1 \
		 ORDER BY accessed_at DESC \
		 LIMIT $2",
	)
	.bind(artifact_id)
	.bind(limit)
	.fetch_all(pool)
	.await?;
    Ok(rows)
}
