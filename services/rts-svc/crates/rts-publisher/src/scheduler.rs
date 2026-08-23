use sqlx::PgPool;
use tracing::{error, info};
use crate::bundle::create_bundle;
use crate::signer::{sign_bundle, upload_to_minio};
use rts_tally::tlog_client::TlogClient;

/// Publication cadence: bundle, sign, upload, then anchor the snapshot into
/// the transparency log via the transactional outbox.
///
/// `tlog` is `None` only when the signing key is not provisioned; every
/// cycle logs that anchoring is disabled so a misconfigured deployment can
/// never be mistaken for a healthy one.
pub async fn run_publication_loop(pool: &PgPool, interval_secs: u64, tlog: Option<TlogClient>) {
    let interval = std::time::Duration::from_secs(interval_secs);
    loop {
        if tlog.is_none() {
            info!("scheduler: transparency-log anchoring disabled (no EEL_TLOG_SIGNING_KEY)");
        }
        match create_bundle(pool).await {
            Ok(bundle) => {
                match sign_bundle(&bundle).await {
                    Ok(hash) => match upload_to_minio(&bundle, &hash).await {
                        Ok(url) => {
                            info!(snapshot_id=%bundle.snapshot_id, url=%url, "scheduler: publication complete");
                            if let Some(client) = &tlog {
                                match client
                                    .append_leaf(pool, &bundle.merkle_root, &bundle.snapshot_id)
                                    .await
                                {
                                    Ok(event_id) => info!(
                                        event_id = %event_id,
                                        snapshot_id = %bundle.snapshot_id,
                                        "scheduler: snapshot anchored"
                                    ),
                                    Err(e) => error!("scheduler: transparency-log anchor failed: {e}"),
                                }
                            }
                        }
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
