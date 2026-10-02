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

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::error::EventError;

struct CacheEntry {
    schema: serde_json::Value,
    fetched_at: Instant,
}

/// Async Karapace schema registry client with TTL-based in-memory cache.
///
/// Fetches JSON Schema documents by subject from a Karapace-compatible
/// Schema Registry HTTP endpoint and caches them to avoid repeated lookups
/// during a service lifetime.
#[derive(Clone)]
pub struct KarapaceClient {
    base_url: String,
    ttl: Duration,
    http: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl KarapaceClient {
    /// Create a new client targeting `base_url` (e.g. `http://karapace:8081`).
    /// Schemas are cached for `ttl`.
    pub fn new(base_url: impl Into<String>, ttl: Duration) -> Self {
        Self {
            base_url: base_url.into(),
            ttl,
            http: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Fetch the latest schema for `subject`, returning a cached copy if
    /// it was fetched within the configured TTL.
    pub async fn get_schema(&self, subject: &str) -> Result<serde_json::Value, EventError> {
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(subject) {
                if entry.fetched_at.elapsed() < self.ttl {
                    return Ok(entry.schema.clone());
                }
            }
        }

        let url = format!("{}/subjects/{}/versions/latest", self.base_url, subject);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| EventError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(EventError::Schema(format!(
                "schema registry returned {} for subject {}",
                resp.status(),
                subject
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| EventError::Http(e.to_string()))?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(
                subject.to_string(),
                CacheEntry {
                    schema: body.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }

        Ok(body)
    }
}
