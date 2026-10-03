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

use crate::error::ResilienceError;
use chrono::{DateTime, Duration, Utc};
use hex::encode as hex_encode;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Entry stored for an idempotency key.
#[derive(Debug, Clone)]
pub struct IdempotencyEntry {
    pub body_hash: String,
    pub response: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

/// In-memory idempotency store (production: use PG + Valkey).
#[derive(Default, Clone)]
pub struct IdempotencyStore {
    ttl: Duration,
    entries: Arc<Mutex<HashMap<String, IdempotencyEntry>>>,
}

impl IdempotencyStore {
    pub fn new(ttl_secs: i64) -> Self {
        Self {
            ttl: Duration::seconds(ttl_secs),
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Compute body hash (SHA-256 hex).
    pub fn body_hash(body: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(body);
        hex_encode(hasher.finalize())
    }

    /// Deduplicate: returns cached response if key was seen before with same body.
    /// Returns `Err(IdempotencyConflict)` if same key, different body.
    /// Returns `Ok(None)` if key is new.
    pub async fn check(
        &self,
        subject: &str,
        idempotency_key: &str,
        body: &[u8],
    ) -> Result<Option<Vec<u8>>, ResilienceError> {
        let store_key = format!("{subject}:{idempotency_key}");
        let hash = Self::body_hash(body);

        let mut guard = self.entries.lock().await;
        // Prune expired
        let cutoff = Utc::now() - self.ttl;
        guard.retain(|_, v| v.created_at > cutoff);

        match guard.get(&store_key) {
            Some(entry) if entry.body_hash == hash => Ok(entry.response.clone()),
            Some(_) => Err(ResilienceError::IdempotencyConflict),
            None => {
                guard.insert(
                    store_key,
                    IdempotencyEntry {
                        body_hash: hash,
                        response: None,
                        created_at: Utc::now(),
                    },
                );
                Ok(None)
            }
        }
    }

    /// Store the response for an idempotency key.
    pub async fn store_response(&self, subject: &str, idempotency_key: &str, response: Vec<u8>) {
        let store_key = format!("{subject}:{idempotency_key}");
        let mut guard = self.entries.lock().await;
        if let Some(entry) = guard.get_mut(&store_key) {
            entry.response = Some(response);
        }
    }
}
