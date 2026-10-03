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

//! List and re-drive parked ingest failures (tlog_ingest_failures).

use sqlx::PgPool;

/// Print all open failures; exit non-zero via return count so CI can gate on
/// an unverified public log.
pub async fn list(db_url: &str) -> anyhow::Result<usize> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(db_url)
        .await?;
    let open = tlog_ingest::failures::list_open(&pool).await?;
    for f in &open {
        println!(
            "{}  {}#{} @{}  event={}  kind={}  {}",
            f.id,
            f.topic,
            f.partition,
            f.kafka_offset,
            f.event_id.as_deref().unwrap_or("-"),
            f.error_kind,
            if f.error_detail.len() > 80 {
                &f.error_detail[..80]
            } else {
                &f.error_detail
            }
        );
    }
    println!("{} open ingest failure(s)", open.len());
    Ok(open.len())
}

/// Mark a failure replayed after a human has confirmed the event is now in
/// the log (e.g. via BulkIngest or manual append).
pub async fn ack(db_url: &str, id: uuid::Uuid) -> anyhow::Result<()> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(db_url)
        .await?;
    tlog_ingest::failures::mark_replayed(&pool, id).await?;
    println!("failure {id} marked replayed");
    Ok(())
}
