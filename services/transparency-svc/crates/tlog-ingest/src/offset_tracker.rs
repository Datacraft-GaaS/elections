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
