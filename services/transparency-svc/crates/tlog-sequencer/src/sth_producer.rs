//! Background tasks that produce STHs at 1s / 60s / 5min cadences.

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::time;
use tlog_domain::SthCadence;
use tracing::info;

pub struct SthProducer {
    pool: Arc<PgPool>,
}

impl SthProducer {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Spawn all three STH background tasks (hot=1s, default=60s, batch=5min).
    pub fn spawn_all(self: Arc<Self>) {
        let p = self.clone();
        tokio::spawn(async move { p.run_cadence(SthCadence::Hot).await });
        let p = self.clone();
        tokio::spawn(async move { p.run_cadence(SthCadence::Default).await });
        let p = self.clone();
        tokio::spawn(async move { p.run_cadence(SthCadence::Batch).await });
    }

    async fn run_cadence(&self, cadence: SthCadence) {
        let interval = Duration::from_secs(cadence.interval_secs());
        let mut ticker = time::interval(interval);
        loop {
            ticker.tick().await;
            if let Err(e) = self.produce_sth(cadence).await {
                tracing::warn!(cadence=?cadence, error=%e, "STH production failed");
            }
        }
    }

    async fn produce_sth(&self, cadence: SthCadence) -> Result<(), sqlx::Error> {
        let row: Option<(i64, Vec<u8>)> = sqlx::query_as(
            "SELECT index, hash FROM leaves ORDER BY index DESC LIMIT 1",
        )
        .fetch_optional(&*self.pool)
        .await?;

        let tree_size = row.as_ref().map(|(i, _)| i + 1).unwrap_or(0);
        let root_hash = row.map(|(_, h)| h).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO sth (id, tree_size, root_hash, cadence, signed_at, signature, key_role, key_fingerprint)
            VALUES (gen_random_uuid(), $1, $2, $3, now(), '\x00'::bytea, 'sth', 'pending-wp22')
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(tree_size)
        .bind(&root_hash)
        .bind(cadence_str(cadence))
        .execute(&*self.pool)
        .await?;

        info!(cadence=?cadence, tree_size=tree_size, "STH produced");
        Ok(())
    }
}

fn cadence_str(c: SthCadence) -> &'static str {
    match c {
        SthCadence::Hot => "hot",
        SthCadence::Default => "default",
        SthCadence::Batch => "batch",
    }
}
