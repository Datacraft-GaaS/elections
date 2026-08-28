//! Background tasks that produce STHs at 1s / 60s / 5min cadences.
//!
//! Every head is signed over the canonical bytes
//! `tree_size || root_hash || cadence` (see [`crate::signer`]) before it is
//! persisted; unsigned placeholder heads are no longer written.

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::time;
use tlog_domain::SthCadence;
use tracing::{info, warn};

use crate::signer::SthSigner;

pub struct SthProducer {
    pool: Arc<PgPool>,
    signer: Arc<SthSigner>,
}

impl SthProducer {
    /// Build a producer. Signing backend is chosen from the environment
    /// (PKCS#11/softHSM by default; explicit dev Ed25519 with
    /// `EEL_STH_SIGNER=dev`). A failure to construct a signer aborts startup:
    /// an unverified log is worse than no log.
    pub fn new(pool: Arc<PgPool>, signer: SthSigner) -> Self {
        Self {
            pool,
            signer: Arc::new(signer),
        }
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
                warn!(cadence=?cadence, error=%e, "STH production failed");
            }
        }
    }

    async fn produce_sth(&self, cadence: SthCadence) -> Result<(), sqlx::Error> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT index FROM leaves ORDER BY index DESC LIMIT 1")
                .fetch_optional(&*self.pool)
                .await?;

        let tree_size = row.as_ref().map(|(i,)| i + 1).unwrap_or(0);
        // The root must be the Merkle root over ALL leaves, not the newest
        // leaf hash; clients verify inclusion proofs against this root.
        let root_hash = crate::proofs::current_root(&self.pool).await?;
        let cadence_s = cadence_str(cadence);

        // Sign BEFORE persisting so no unsigned head ever exists on disk.
        let (signature, fingerprint) = self.signer.sign_sth(tree_size as u64, &root_hash, cadence_s)
            .map_err(|e| {
                tracing::error!(error=%e, "STH signing failed");
                sqlx::Error::Configuration(Box::new(std::io::Error::other(e.to_string())))
            })?;

        sqlx::query(
            r#"
            INSERT INTO sth (id, tree_size, root_hash, cadence, signed_at, signature, key_role, key_fingerprint)
            VALUES (gen_random_uuid(), $1, $2, $3, now(), $4, 'sth', $5)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(tree_size)
        .bind(&root_hash)
        .bind(cadence_s)
        .bind(&signature)
        .bind(&fingerprint)
        .execute(&*self.pool)
        .await?;

        info!(cadence=?cadence, tree_size=tree_size, key=%fingerprint, "STH produced and signed");
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
