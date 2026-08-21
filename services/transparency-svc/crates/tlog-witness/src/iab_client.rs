//! IAB internal witness: polls the STH endpoint and cosigns via local key (staging)
//! or HSM (production, wired in WP-2.2).

use std::sync::Arc;

use tlog_domain::SignedTreeHead;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

pub struct IabWitnessClient {
    api_base_url: String,
    witness_id: String,
}

impl IabWitnessClient {
    pub fn new(api_base_url: impl Into<String>, witness_id: impl Into<String>) -> Self {
        Self {
            api_base_url: api_base_url.into(),
            witness_id: witness_id.into(),
        }
    }

    /// Start the cosign loop (polls every 60s).
    pub async fn run(self: Arc<Self>) {
        let mut ticker = interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            if let Err(e) = self.cosign_latest().await {
                warn!(error=%e, "IAB witness cosign failed");
            }
        }
    }

    async fn cosign_latest(&self) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/transparency/sth/latest", self.api_base_url);
        let sth: SignedTreeHead = client.get(&url).send().await?.json().await?;

        // TODO(WP-2.2): replace with HSM cosign.
        let sig = format!("staging-cosig-{}", self.witness_id);
        let cosign_url = format!(
            "{}/api/v1/transparency/witnesses/cosign",
            self.api_base_url
        );
        let body = serde_json::json!({
            "sth_id": sth.id,
            "witness_id": self.witness_id,
            "signature": sig,
        });
        client.post(&cosign_url).json(&body).send().await?;
        info!(tree_size = sth.tree_size, witness_id = %self.witness_id, "cosigned STH");
        Ok(())
    }
}
