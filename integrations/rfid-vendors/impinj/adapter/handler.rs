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

// This file is included as a module from src/adapter.rs.
// Direct path: integrations/rfid-vendors/impinj/adapter/handler.rs

use std::time::Duration;

use eel_resilience_rs::CircuitBreaker;
use reqwest::Client;
use tracing::{error, info, warn};

use crate::adapter::{AdapterError, ChallengeResponse, ReadResult, RegistrationResult};

/// HTTP adapter for the py/rfid-reader-driver Impinj endpoint.
///
/// Reads `EEL_IMPINJ_ADAPTER_URL` from the environment (default: http://localhost:8001).
/// All requests go through a circuit breaker (threshold=5, window=60s, cooldown=30s).
pub struct Handler {
    client: Client,
    base_url: String,
    breaker: CircuitBreaker,
}

impl Handler {
    /// Construct from environment or explicit URL.
    pub fn from_env() -> Self {
        let base_url = std::env::var("EEL_IMPINJ_ADAPTER_URL")
            .unwrap_or_else(|_| "http://localhost:8001".to_string());
        Self::new(base_url)
    }

    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let breaker = CircuitBreaker::new(
            "impinj-adapter",
            5,
            Duration::from_secs(60),
            Duration::from_secs(30),
        );
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self { client, base_url, breaker }
    }

    /// Register a reader with the Impinj adapter service.
    pub async fn register(
        &self,
        reader_id: &str,
        svid: &str,
        location: &str,
    ) -> Result<RegistrationResult, AdapterError> {
        self.breaker
            .allow()
            .await
            .map_err(|_| AdapterError::CircuitOpen)?;

        let payload = serde_json::json!({
            "reader_id": reader_id,
            "svid": svid,
            "location": location,
            "reader_type": "impinj",
        });

        let resp = self
            .client
            .post(format!("{}/reader/register", self.base_url))
            .json(&payload)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                self.breaker.record_success().await;
                info!(reader_id, "impinj register ok");
                Ok(RegistrationResult { reader_id: reader_id.to_string() })
            }
            Ok(r) => {
                let status = r.status().as_u16();
                self.breaker.record_failure().await;
                warn!(reader_id, status, "impinj register upstream error");
                Err(AdapterError::Upstream { status })
            }
            Err(e) => {
                self.breaker.record_failure().await;
                error!(reader_id, err = %e, "impinj register network error");
                Err(AdapterError::Network(e.to_string()))
            }
        }
    }

    /// Submit a single tag read.
    pub async fn submit_read(
        &self,
        reader_id: &str,
        tag_epc: &str,
        tamper_indicator: bool,
    ) -> Result<ReadResult, AdapterError> {
        self.breaker
            .allow()
            .await
            .map_err(|_| AdapterError::CircuitOpen)?;

        let payload = serde_json::json!({
            "tag_epc": tag_epc,
            "tamper_indicator": tamper_indicator,
        });

        let resp = self
            .client
            .post(format!("{}/reader/{}/read", self.base_url, reader_id))
            .json(&payload)
            .send()
            .await;

        self.handle_read_response(resp, tag_epc).await
    }

    /// Issue a Gen2v2 HMAC challenge for a tag.
    pub async fn issue_challenge(
        &self,
        reader_id: &str,
        tag_epc: &str,
    ) -> Result<ChallengeResponse, AdapterError> {
        self.breaker
            .allow()
            .await
            .map_err(|_| AdapterError::CircuitOpen)?;

        let payload = serde_json::json!({ "tag_epc": tag_epc });

        let resp = self
            .client
            .post(format!("{}/reader/{}/challenge", self.base_url, reader_id))
            .json(&payload)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                self.breaker.record_success().await;
                let body: serde_json::Value = r.json().await.map_err(|e| {
                    AdapterError::Network(format!("challenge decode: {e}"))
                })?;
                Ok(ChallengeResponse {
                    nonce_hex: body["nonce_hex"].as_str().unwrap_or("").to_string(),
                    challenge_hex: body["challenge_hex"].as_str().unwrap_or("").to_string(),
                })
            }
            Ok(r) => {
                self.breaker.record_failure().await;
                Err(AdapterError::Upstream { status: r.status().as_u16() })
            }
            Err(e) => {
                self.breaker.record_failure().await;
                Err(AdapterError::Network(e.to_string()))
            }
        }
    }

    /// Query reader health and last-read info.
    pub async fn status(&self, reader_id: &str) -> Result<serde_json::Value, AdapterError> {
        self.breaker
            .allow()
            .await
            .map_err(|_| AdapterError::CircuitOpen)?;

        let resp = self
            .client
            .get(format!("{}/reader/{}/status", self.base_url, reader_id))
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                self.breaker.record_success().await;
                r.json().await.map_err(|e| AdapterError::Network(e.to_string()))
            }
            Ok(r) => {
                self.breaker.record_failure().await;
                Err(AdapterError::Upstream { status: r.status().as_u16() })
            }
            Err(e) => {
                self.breaker.record_failure().await;
                Err(AdapterError::Network(e.to_string()))
            }
        }
    }

    async fn handle_read_response(
        &self,
        resp: Result<reqwest::Response, reqwest::Error>,
        tag_epc: &str,
    ) -> Result<ReadResult, AdapterError> {
        match resp {
            Ok(r) if r.status().is_success() => {
                self.breaker.record_success().await;
                let body: serde_json::Value =
                    r.json().await.map_err(|e| AdapterError::Network(e.to_string()))?;
                Ok(ReadResult {
                    tag_epc: tag_epc.to_string(),
                    accepted: body["accepted"].as_bool().unwrap_or(false),
                    tamper_detected: body
                        .get("tag_state")
                        .and_then(|s| s.as_str())
                        .map(|s| s == "tampered")
                        .unwrap_or(false),
                })
            }
            Ok(r) => {
                self.breaker.record_failure().await;
                Err(AdapterError::Upstream { status: r.status().as_u16() })
            }
            Err(e) => {
                self.breaker.record_failure().await;
                Err(AdapterError::Network(e.to_string()))
            }
        }
    }
}
