use sqlx::PgPool;
use tracing::{error, info};
use crate::bundle::create_bundle;
use crate::signer::{sign_bundle, upload_to_minio};

pub async fn run_publication_loop(pool: &PgPool, interval_secs: u64) {
    let interval = std::time::Duration::from_secs(interval_secs);
    loop {
        match create_bundle(pool).await {
            Ok(bundle) => {
                match sign_bundle(&bundle).await {
                    Ok(hash) => match upload_to_minio(&bundle, &hash).await {
                        Ok(url) => info!(snapshot_id=%bundle.snapshot_id, url=%url, "scheduler: publication complete"),
                        Err(e) => error!("scheduler: upload failed: {e}"),
                    },
                    Err(e) => error!("scheduler: signing failed: {e}"),
                }
            }
            Err(crate::bundle::BundleError::NoData) => info!("scheduler: no compiled form34c yet, skipping"),
            Err(e) => error!("scheduler: bundle creation failed: {e}"),
        }
        tokio::time::sleep(interval).await;
    }
}
