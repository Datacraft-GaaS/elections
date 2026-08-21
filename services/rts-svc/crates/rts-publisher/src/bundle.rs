use chrono::{DateTime, Utc};
use rts_domain::aggregates::form34c::{Form34C, Form34CStatus};
use sqlx::PgPool;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BundleError {
    #[error("bundle db error: {0}")] Db(#[from] sqlx::Error),
    #[error("bundle serialization error: {0}")] Serde(#[from] serde_json::Error),
    #[error("no published form34c found")] NoData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublicationBundle {
    pub snapshot_id: String,
    pub form34c: Form34C,
    pub merkle_root: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_bundle(pool: &PgPool) -> Result<PublicationBundle, BundleError> {
    let row: Option<(uuid::Uuid, i64, serde_json::Value, i64, i64, i64, i64, serde_json::Value, Option<String>, String, Option<DateTime<Utc>>)> =
        sqlx::query_as(
            "SELECT stream_id, stream_version, compiled_constituencies, total_registered, total_valid, total_rejected, total_spoilt, candidate_totals, merkle_root, status, published_at FROM form34c_view WHERE status IN ('compiled','published') ORDER BY updated_at DESC LIMIT 1"
        ).fetch_optional(pool).await?;
    let row = row.ok_or(BundleError::NoData)?;
    let (stream_id, stream_version, compiled_constituencies, total_registered, total_valid, total_rejected, total_spoilt, candidate_totals, merkle_root_opt, _status, published_at) = row;
    let form34c = Form34C {
        stream_id, version: stream_version,
        compiled_constituencies: serde_json::from_value(compiled_constituencies)?,
        total_registered: total_registered as u64, total_valid: total_valid as u64,
        total_rejected: total_rejected as u64, total_spoilt: total_spoilt as u64,
        candidate_totals: serde_json::from_value(candidate_totals)?,
        merkle_root: merkle_root_opt.clone(), status: Form34CStatus::Compiled, published_at,
    };
    let merkle_root = merkle_root_opt.unwrap_or_default();
    let snapshot_id = Uuid::new_v4().to_string();
    info!(snapshot_id, merkle_root, "bundle: created publication bundle");
    Ok(PublicationBundle { snapshot_id, form34c, merkle_root, created_at: Utc::now() })
}
