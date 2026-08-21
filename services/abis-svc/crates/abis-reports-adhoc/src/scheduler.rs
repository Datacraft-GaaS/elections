use std::sync::Arc;

use tokio::time::{interval, Duration};

/// Runs as a background tokio task. Purges expired previews every 5 minutes.
pub async fn run_preview_purger(pool: Arc<sqlx::PgPool>) {
    let mut tick = interval(Duration::from_secs(5 * 60));
    loop {
        tick.tick().await;
        match crate::repository::purge_expired_previews(&pool).await {
            Ok(n) => tracing::info!("purged {n} expired previews"),
            Err(e) => tracing::error!("preview purge error: {e}"),
        }
    }
}
