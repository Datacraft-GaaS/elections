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

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// service.yaml schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceManifest {
    /// Service name, e.g. "rts-svc"
    pub name: String,
    /// OCI image reference (without tag), e.g. "harbor.eel.internal/eel/rts-svc"
    pub image: String,
    /// Artifacts produced/consumed by this service (optional)
    #[serde(default)]
    pub artifacts: Vec<String>,
    /// Sites this service should be deployed to; defaults to all 7 if empty
    #[serde(default)]
    pub sites: Vec<String>,
    /// Deployment strategy per site (optional overrides)
    #[serde(default)]
    pub strategy: BTreeMap<String, String>,
    /// Replica counts per site (optional overrides)
    #[serde(default)]
    pub replicas: BTreeMap<String, u32>,
}

impl ServiceManifest {
    pub fn effective_sites(&self) -> Vec<String> {
        if self.sites.is_empty() {
            all_sites()
        } else {
            self.sites.clone()
        }
    }
}

pub fn all_sites() -> Vec<String> {
    vec![
        "primary".to_string(),
        "mirror".to_string(),
        "edge-nairobi".to_string(),
        "edge-mombasa".to_string(),
        "edge-kisumu".to_string(),
        "edge-eldoret".to_string(),
        "edge-garissa".to_string(),
    ]
}

pub fn all_canonical_services() -> Vec<String> {
    vec![
        "biometric-edge-svc",
        "bvr-svc",
        "abis-svc",
        "vprs-svc",
        "bvi-svc",
        "rts-svc",
        "portal-svc",
        "crms-svc",
        "transparency-svc",
        "ecc-svc",
        "boundary-svc",
        "queue-svc",
        "inventory-svc",
        "governance-svc",
        "ai-platform-svc",
        "field-mobile-svc",
        "training-svc",
        "support-admin-svc",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

pub fn load_service(name: &str) -> Result<ServiceManifest> {
    let path = format!("services/{}/deploy/service.yaml", name);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parsing {}", path))
}

pub fn discover_services() -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in glob::glob("services/*/deploy/service.yaml")
        .context("glob services/*/deploy/service.yaml")?
        .flatten()
    {
        if let Some(name) = entry
            .components()
            .nth(1)
            .and_then(|c| c.as_os_str().to_str())
        {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

// ---------------------------------------------------------------------------
// Topic manifest schema (platform/kafka/topics/<topic>.yaml)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicManifest {
    pub name: String,
    /// Kafka topic name, e.g. "eel.rts.results.v1"
    pub topic: String,
    #[serde(default)]
    pub partitions: u32,
    #[serde(default)]
    pub replication_factor: u32,
    /// SPIFFE IDs allowed to produce (e.g. "spiffe://eel.prod/ns/rts-svc/sa/rts-producer")
    #[serde(default)]
    pub producers: Vec<String>,
    /// SPIFFE IDs allowed to consume
    #[serde(default)]
    pub consumers: Vec<String>,
}
