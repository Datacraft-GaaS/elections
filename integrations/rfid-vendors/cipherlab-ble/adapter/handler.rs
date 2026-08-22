// Direct path: integrations/rfid-vendors/cipherlab-ble/adapter/handler.rs
// Included via #[path] from src/adapter.rs.

use std::time::Duration;

use eel_resilience_rs::CircuitBreaker;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::{error, info, warn};

use crate::adapter::{AdapterError, ChallengeResponse, ReadResult, RegistrationResult};

/// HTTP adapter for the py/rfid-reader-driver CipherLab BLE endpoint.
///
/// Reads `EEL_CIPHERLAB_BLE_ADAPTER_URL` (default: http://localhost:8003).
/// The Python driver uses bleak (MIT-licensed) to communicate with the
/// CipherLab RS36 via BLE + EPC Gen2 GATT profile.  No commercial SDK required.
pub struct Handler {
    client: Client,
    base_url: String,
    breaker: CircuitBreaker,
}

impl Handler {
    pub fn from_env() -> Self {
        let base_url = std::env::var("EEL_CIPHERLAB_BLE_ADAPTER_URL")
            .unwrap_or_else(|_| "http://localhost:8003".to_string());
        Self::new(base_url)
    }

    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let breaker = CircuitBreaker::new(
            "cipherlab-ble-adapter",
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

    pub async fn register(
        &self,
        reader_id: &str,
        svid: &str,
        location: &str,
    ) -> Result<RegistrationResult, AdapterError> {
        self.breaker.allow().await.map_err(|_| AdapterError::CircuitOpen)?;
        let payload = json!({
            "reader_id": reader_id,
            "svid": svid,
            "location": location,
            "reader_type": "cipherlab",
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
                info!(reader_id, "cipherlab-ble register ok");
                Ok(RegistrationResult { reader_id: reader_id.to_string() })
            }
            Ok(r) => {
                self.breaker.record_failure().await;
                warn!(reader_id, status = r.status().as_u16(), "cipherlab-ble register error");
                Err(AdapterError::Upstream { status: r.status().as_u16() })
            }
            Err(e) => {
                self.breaker.record_failure().await;
                error!(reader_id, err = %e, "cipherlab-ble network error");
                Err(AdapterError::Network(e.to_string()))
            }
        }
    }

    pub async fn submit_read(
        &self,
        reader_id: &str,
        tag_epc: &str,
        tamper_indicator: bool,
    ) -> Result<ReadResult, AdapterError> {
        self.breaker.allow().await.map_err(|_| AdapterError::CircuitOpen)?;
        let payload = json!({ "tag_epc": tag_epc, "tamper_indicator": tamper_indicator });
        let resp = self
            .client
            .post(format!("{}/reader/{}/read", self.base_url, reader_id))
            .json(&payload)
            .send()
            .await;
        match resp {
            Ok(r) if r.status().is_success() => {
                self.breaker.record_success().await;
                let body: Value = r.json().await.map_err(|e| AdapterError::Network(e.to_string()))?;
                Ok(ReadResult {
                    tag_epc: tag_epc.to_string(),
                    accepted: body["accepted"].as_bool().unwrap_or(false),
                    tamper_detected: body.get("tag_state")
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

    pub async fn issue_challenge(
        &self,
        reader_id: &str,
        tag_epc: &str,
    ) -> Result<ChallengeResponse, AdapterError> {
        self.breaker.allow().await.map_err(|_| AdapterError::CircuitOpen)?;
        let payload = json!({ "tag_epc": tag_epc });
        let resp = self
            .client
            .post(format!("{}/reader/{}/challenge", self.base_url, reader_id))
            .json(&payload)
            .send()
            .await;
        match resp {
            Ok(r) if r.status().is_success() => {
                self.breaker.record_success().await;
                let body: Value = r.json().await.map_err(|e| AdapterError::Network(e.to_string()))?;
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

    pub async fn status(&self, reader_id: &str) -> Result<Value, AdapterError> {
        self.breaker.allow().await.map_err(|_| AdapterError::CircuitOpen)?;
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
}
