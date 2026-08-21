use eel_eventstore_rs::StoredEvent;
use rts_domain::events::{EVENT_FORM34B_COMPILED, EVENT_FORM34C_COMPILED};
use sqlx::PgPool;
use tracing::info;
use crate::projector::ProjectionError;

pub struct CroViewProjector { pub pool: PgPool }
impl CroViewProjector {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn project(&self, event: &StoredEvent) -> Result<(), ProjectionError> {
        match event.event_type.as_str() {
            EVENT_FORM34B_COMPILED => self.upsert_34b(event).await,
            EVENT_FORM34C_COMPILED => self.upsert_34c(event).await,
            _ => Ok(()),
        }
    }
    async fn upsert_34b(&self, event: &StoredEvent) -> Result<(), ProjectionError> {
        let p = &event.payload;
        let constituency_code = p.get("constituency_code").and_then(|v| v.as_str()).unwrap_or_default();
        let county_code = p.get("county_code").and_then(|v| v.as_str()).unwrap_or_default();
        let compiled_stations = p.get("compiled_stations").cloned().unwrap_or(serde_json::json!([]));
        let total_registered = p.get("total_registered").and_then(|v| v.as_i64()).unwrap_or(0);
        let total_valid = p.get("total_valid").and_then(|v| v.as_i64()).unwrap_or(0);
        let total_rejected = p.get("total_rejected").and_then(|v| v.as_i64()).unwrap_or(0);
        let total_spoilt = p.get("total_spoilt").and_then(|v| v.as_i64()).unwrap_or(0);
        let candidate_totals = p.get("candidate_totals").cloned().unwrap_or(serde_json::json!([]));
        sqlx::query(
            "INSERT INTO form34b_view (stream_id, stream_version, constituency_code, county_code, compiled_stations, total_registered, total_valid, total_rejected, total_spoilt, candidate_totals, status)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'compiled')
             ON CONFLICT (stream_id) DO UPDATE SET stream_version=EXCLUDED.stream_version, compiled_stations=EXCLUDED.compiled_stations, total_registered=EXCLUDED.total_registered, total_valid=EXCLUDED.total_valid, total_rejected=EXCLUDED.total_rejected, total_spoilt=EXCLUDED.total_spoilt, candidate_totals=EXCLUDED.candidate_totals, status='compiled', updated_at=now()"
        )
        .bind(event.stream_id).bind(event.stream_version).bind(constituency_code).bind(county_code)
        .bind(compiled_stations).bind(total_registered).bind(total_valid).bind(total_rejected).bind(total_spoilt).bind(candidate_totals)
        .execute(&self.pool).await?;
        info!(stream_id=%event.stream_id, constituency_code, "cro_view: upserted form34b_view");
        Ok(())
    }
    async fn upsert_34c(&self, event: &StoredEvent) -> Result<(), ProjectionError> {
        let p = &event.payload;
        let compiled_constituencies = p.get("compiled_constituencies").cloned().unwrap_or(serde_json::json!([]));
        let total_registered = p.get("total_registered").and_then(|v| v.as_i64()).unwrap_or(0);
        let total_valid = p.get("total_valid").and_then(|v| v.as_i64()).unwrap_or(0);
        let total_rejected = p.get("total_rejected").and_then(|v| v.as_i64()).unwrap_or(0);
        let total_spoilt = p.get("total_spoilt").and_then(|v| v.as_i64()).unwrap_or(0);
        let candidate_totals = p.get("candidate_totals").cloned().unwrap_or(serde_json::json!([]));
        let merkle_root: Option<String> = p.get("merkle_root").and_then(|v| v.as_str()).map(|s| s.to_string());
        sqlx::query(
            "INSERT INTO form34c_view (stream_id, stream_version, compiled_constituencies, total_registered, total_valid, total_rejected, total_spoilt, candidate_totals, merkle_root, status)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'compiled')
             ON CONFLICT (stream_id) DO UPDATE SET stream_version=EXCLUDED.stream_version, compiled_constituencies=EXCLUDED.compiled_constituencies, total_registered=EXCLUDED.total_registered, total_valid=EXCLUDED.total_valid, total_rejected=EXCLUDED.total_rejected, total_spoilt=EXCLUDED.total_spoilt, candidate_totals=EXCLUDED.candidate_totals, merkle_root=EXCLUDED.merkle_root, status='compiled', updated_at=now()"
        )
        .bind(event.stream_id).bind(event.stream_version).bind(compiled_constituencies)
        .bind(total_registered).bind(total_valid).bind(total_rejected).bind(total_spoilt).bind(candidate_totals).bind(merkle_root)
        .execute(&self.pool).await?;
        info!(stream_id=%event.stream_id, "cro_view: upserted form34c_view");
        Ok(())
    }
}
