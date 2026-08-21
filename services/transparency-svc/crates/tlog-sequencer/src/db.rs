//! Database access layer for the sequencer.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tlog_domain::{Domain, LeafHash};
use uuid::Uuid;

/// A leaf row as stored in PostgreSQL.
#[derive(Debug, sqlx::FromRow)]
pub struct LeafRow {
    pub id: Uuid,
    pub index: i64,
    pub domain: String,
    pub hash: Vec<u8>,
    pub producer_spiffe_id: String,
    pub source_event_id: String,
    pub source_topic: String,
    pub event_time: DateTime<Utc>,
    pub appended_at: DateTime<Utc>,
    pub producer_signature: Vec<u8>,
}

/// Append a leaf and return its assigned index.
pub async fn append_leaf(
    pool: &PgPool,
    domain: Domain,
    hash: &LeafHash,
    producer_spiffe_id: &str,
    source_event_id: &str,
    source_topic: &str,
    event_time: DateTime<Utc>,
    producer_signature: &[u8],
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO leaves (id, domain, hash, producer_spiffe_id,
            source_event_id, source_topic, event_time, appended_at, producer_signature)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, now(), $7)
        RETURNING index
        "#,
    )
    .bind(domain.as_str())
    .bind(hash.as_ref())
    .bind(producer_spiffe_id)
    .bind(source_event_id)
    .bind(source_topic)
    .bind(event_time)
    .bind(producer_signature)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Check for an existing leaf by (source_topic, source_event_id) for deduplication.
pub async fn leaf_exists(
    pool: &PgPool,
    source_topic: &str,
    source_event_id: &str,
) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM leaves WHERE source_topic=$1 AND source_event_id=$2)",
    )
    .bind(source_topic)
    .bind(source_event_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Fetch the current tree size (max index + 1).
pub async fn tree_size(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (Option<i64>,) = sqlx::query_as("SELECT MAX(index) + 1 FROM leaves")
        .fetch_one(pool)
        .await?;
    Ok(row.0.unwrap_or(0))
}
