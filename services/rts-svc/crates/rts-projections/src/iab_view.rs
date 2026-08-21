use eel_eventstore_rs::StoredEvent;
use sqlx::PgPool;
use tracing::info;
use crate::projector::ProjectionError;

pub struct IabViewProjector { pub pool: PgPool }
impl IabViewProjector {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn project(&self, event: &StoredEvent) -> Result<(), ProjectionError> {
        if event.event_type == "eel.rts.tally_published.v1" {
            info!(stream_id=%event.stream_id, "iab_view: tally_published event received (stub)");
        }
        Ok(())
    }
}
