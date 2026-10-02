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

use eel_eventstore_rs::StoredEvent;
use rts_domain::events::EVENT_TALLY_PUBLISHED;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;
use crate::projector::ProjectionError;

pub struct PortalSnapshotProjector { pub pool: PgPool }
impl PortalSnapshotProjector {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn project(&self, event: &StoredEvent) -> Result<(), ProjectionError> {
        if event.event_type != EVENT_TALLY_PUBLISHED { return Ok(()); }
        let snapshot_id = Uuid::new_v4();
        let merkle_root = event.payload.get("merkle_root").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let signed_url: Option<String> = event.payload.get("signed_url").and_then(|v| v.as_str()).map(|s| s.to_string());
        let tlog_leaf_index: Option<i64> = event.payload.get("tlog_leaf_index").and_then(|v| v.as_i64());
        sqlx::query("INSERT INTO publication_snapshots (snapshot_id, merkle_root, signed_url, tlog_leaf_index) VALUES ($1,$2,$3,$4)")
            .bind(snapshot_id).bind(&merkle_root).bind(signed_url).bind(tlog_leaf_index)
            .execute(&self.pool).await?;
        info!(snapshot_id=%snapshot_id, merkle_root, "portal_snapshot: recorded");
        Ok(())
    }
}
