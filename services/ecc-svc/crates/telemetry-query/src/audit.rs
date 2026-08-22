//! Audit log: every query recorded in `eel_telemetry_queries`.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::dsl::TelemetryQuery;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryAuditRecord {
    pub query_id: Uuid,
    pub actor_svid: String,
    pub query_dsl: serde_json::Value,
    pub executed_at: chrono::DateTime<Utc>,
    pub row_count: i64,
    pub result_sha256: Option<String>,
    pub evidence_exported: bool,
    pub preset_name: Option<String>,
    pub nl_text: Option<String>,
}

/// Insert a query audit record into `eel_telemetry_queries`.
///
/// The table is append-only by convention (trigger blocks UPDATE/DELETE).
pub async fn record_query(
    pool: &PgPool,
    actor_svid: &str,
    query: &TelemetryQuery,
    row_count: i64,
    result_sha256: Option<&str>,
    preset_name: Option<&str>,
    nl_text: Option<&str>,
) -> Result<Uuid, AuditError> {
    let query_id = Uuid::now_v7();
    let dsl = serde_json::to_value(query)?;

    sqlx::query(
        "INSERT INTO eel_telemetry_queries \
         (query_id, actor_svid, query_dsl, executed_at, row_count, \
          result_sha256, evidence_exported, preset_name, nl_text) \
         VALUES ($1, $2, $3, $4, $5, $6, false, $7, $8)",
    )
    .bind(query_id)
    .bind(actor_svid)
    .bind(dsl)
    .bind(Utc::now())
    .bind(row_count)
    .bind(result_sha256)
    .bind(preset_name)
    .bind(nl_text)
    .execute(pool)
    .await?;

    Ok(query_id)
}

/// Mark an existing audit record as having had evidence exported.
pub async fn mark_evidence_exported(
    pool: &PgPool,
    query_id: Uuid,
    bundle_sha256: &str,
) -> Result<(), AuditError> {
    sqlx::query(
        "UPDATE eel_telemetry_queries \
         SET evidence_exported = true, result_sha256 = $2 \
         WHERE query_id = $1",
    )
    .bind(query_id)
    .bind(bundle_sha256)
    .execute(pool)
    .await?;
    Ok(())
}
