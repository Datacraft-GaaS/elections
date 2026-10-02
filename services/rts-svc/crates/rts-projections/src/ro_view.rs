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
use rts_domain::events::{EVENT_FORM34A_AMENDED, EVENT_FORM34A_CAPTURED, EVENT_FORM34A_VERIFIED};
use sqlx::PgPool;
use tracing::info;
use crate::projector::ProjectionError;

pub struct RoViewProjector { pub pool: PgPool }
impl RoViewProjector {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn project(&self, event: &StoredEvent) -> Result<(), ProjectionError> {
        match event.event_type.as_str() {
            EVENT_FORM34A_CAPTURED | EVENT_FORM34A_VERIFIED | EVENT_FORM34A_AMENDED => self.upsert(event).await,
            _ => Ok(()),
        }
    }
    async fn upsert(&self, event: &StoredEvent) -> Result<(), ProjectionError> {
        let p = &event.payload;
        let station_code = p.get("station_code").and_then(|v| v.as_str()).unwrap_or_default();
        let constituency_code = p.get("constituency_code").and_then(|v| v.as_str()).unwrap_or_default();
        let county_code = p.get("county_code").and_then(|v| v.as_str()).unwrap_or_default();
        let registered_voters = p.get("registered_voters").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let votes_cast = p.get("votes_cast").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let valid_votes = p.get("valid_votes").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let rejected_votes = p.get("rejected_votes").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let spoilt_ballots = p.get("spoilt_ballots").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let candidates = p.get("candidates").cloned().unwrap_or(serde_json::json!([]));
        let status = match event.event_type.as_str() {
            EVENT_FORM34A_CAPTURED => "captured", EVENT_FORM34A_VERIFIED => "verified",
            EVENT_FORM34A_AMENDED => "amended", _ => "draft",
        };
        sqlx::query(
            "INSERT INTO form34a_view (form_id, stream_id, stream_version, station_code, constituency_code, county_code, registered_voters, votes_cast, valid_votes, rejected_votes, spoilt_ballots, candidate_totals, status)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
             ON CONFLICT (form_id) DO UPDATE SET stream_version=EXCLUDED.stream_version, valid_votes=EXCLUDED.valid_votes, rejected_votes=EXCLUDED.rejected_votes, spoilt_ballots=EXCLUDED.spoilt_ballots, votes_cast=EXCLUDED.votes_cast, candidate_totals=EXCLUDED.candidate_totals, status=EXCLUDED.status, updated_at=now()"
        )
        .bind(event.id).bind(event.stream_id).bind(event.stream_version)
        .bind(station_code).bind(constituency_code).bind(county_code)
        .bind(registered_voters).bind(votes_cast).bind(valid_votes).bind(rejected_votes).bind(spoilt_ballots)
        .bind(candidates).bind(status)
        .execute(&self.pool).await?;
        info!(form_id=%event.id, event_type=%event.event_type, "ro_view: upserted form34a_view");
        Ok(())
    }
}
