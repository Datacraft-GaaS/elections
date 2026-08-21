//! Tracks the last-committed Kafka offset per partition in PostgreSQL.
//! Enables catch-up after log downtime.

use sqlx::PgPool;

pub async fn save_offset(
    pool: &PgPool,
    topic: &str,
    partition: i32,
    offset: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO ingest_offsets (topic, partition, committed_offset, updated_at)
        VALUES ($1, $2, $3, now())
        ON CONFLICT (topic, partition)
        DO UPDATE SET committed_offset = EXCLUDED.committed_offset, updated_at = now()
        "#,
    )
    .bind(topic)
    .bind(partition)
    .bind(offset)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn load_offset(
    pool: &PgPool,
    topic: &str,
    partition: i32,
) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT committed_offset FROM ingest_offsets WHERE topic=$1 AND partition=$2",
    )
    .bind(topic)
    .bind(partition)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(o,)| o))
}
