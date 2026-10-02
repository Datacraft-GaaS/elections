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

use std::sync::Arc;

use anyhow::Context;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::stix::{parse_bundle, StixBundle};

#[derive(Debug, Clone)]
pub struct TaxiiEndpoint {
    pub url: String,
    pub collection_id: String,
    pub api_root: String,
    pub label: String,
}

pub struct FeedManager {
    pub client: reqwest::Client,
    pub endpoints: Vec<TaxiiEndpoint>,
    pub bundles: Arc<RwLock<Vec<StixBundle>>>,
}

impl FeedManager {
    pub fn new(endpoints: Vec<TaxiiEndpoint>) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoints,
            bundles: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn default_endpoints() -> Vec<TaxiiEndpoint> {
        vec![
            TaxiiEndpoint {
                url: "https://cti-taxii.mitre.org/taxii/".into(),
                collection_id: "95ecc380-afe9-11e4-9b6c-751b66dd541e".into(),
                api_root: "stix".into(),
                label: "MITRE ATT&CK Enterprise".into(),
            },
            TaxiiEndpoint {
                url: "https://otx.alienvault.com/taxii/".into(),
                collection_id: "alienvault-otx".into(),
                api_root: "api/v1".into(),
                label: "AlienVault OTX".into(),
            },
            TaxiiEndpoint {
                url: "https://feeds.elections-security.example/taxii/".into(),
                collection_id: "elections-security".into(),
                api_root: "stix".into(),
                label: "Elections Security Feed".into(),
            },
        ]
    }

    pub async fn fetch_all(&self) -> Result<(), anyhow::Error> {
        for endpoint in &self.endpoints {
            let fetch_url = format!(
                "{}/api/{}/{}/objects/",
                endpoint.url.trim_end_matches('/'),
                endpoint.api_root,
                endpoint.collection_id
            );
            info!(label = %endpoint.label, url = %fetch_url, "fetching STIX bundle");

            let resp = self
                .client
                .get(&fetch_url)
                .header("Accept", "application/stix+json;version=2.1")
                .send()
                .await
                .with_context(|| format!("GET {} failed", fetch_url))?;

            if !resp.status().is_success() {
                warn!(
                    label = %endpoint.label,
                    status = %resp.status(),
                    "non-2xx response from TAXII endpoint"
                );
                continue;
            }

            let body = resp.text().await.with_context(|| "reading response body")?;
            match parse_bundle(&body) {
                Ok(bundle) => {
                    let mut lock = self.bundles.write().await;
                    lock.push(bundle);
                }
                Err(e) => {
                    warn!(label = %endpoint.label, error = %e, "failed to parse STIX bundle");
                }
            }
        }
        Ok(())
    }

    pub async fn bundles(&self) -> Vec<StixBundle> {
        self.bundles.read().await.clone()
    }

    pub async fn refresh_loop(&self, interval_secs: u64) {
        loop {
            if let Err(e) = self.fetch_all().await {
                warn!(error = %e, "fetch_all error in refresh loop");
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
        }
    }
}
