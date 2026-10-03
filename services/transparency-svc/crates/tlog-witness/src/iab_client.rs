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

//! IAB internal witness: polls the STH endpoint and cosigns every new tree
//! head with a local Ed25519 key (staging) or an HSM-held key (production;
//! provision the same 32-byte seed via the HSM signer crate in WP-2.2).
//!
//! Cosignature protocol (transparency-svc docs, Appendix D):
//! `Ed25519(concat(tree_size_le64, root_hash_bytes))`, submitted hex-encoded
//! to `POST /api/v1/transparency/witnesses/cosign`.

use std::sync::Arc;

use ed25519_dalek::{Signer, SigningKey};
use tlog_domain::SignedTreeHead;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

pub struct IabWitnessClient {
    api_base_url: String,
    witness_id: String,
    signing_key: SigningKey,
}

impl IabWitnessClient {
    pub fn new(
        api_base_url: impl Into<String>,
        witness_id: impl Into<String>,
        signing_key: SigningKey,
    ) -> Self {
        Self {
            api_base_url: api_base_url.into(),
            witness_id: witness_id.into(),
            signing_key,
        }
    }

    /// Construct a client whose key comes from `EEL_WITNESS_SIGNING_KEY`
    /// (32-byte Ed25519 seed, hex encoded).
    pub fn from_env(
        api_base_url: impl Into<String>,
        witness_id: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let seed_hex = std::env::var("EEL_WITNESS_SIGNING_KEY")
            .map_err(|_| anyhow::anyhow!("EEL_WITNESS_SIGNING_KEY is not set"))?;
        let seed = hex::decode(seed_hex.trim())
            .map_err(|e| anyhow::anyhow!("EEL_WITNESS_SIGNING_KEY is not hex: {e}"))?;
        let seed: [u8; 32] = seed
            .try_into()
            .map_err(|_| anyhow::anyhow!("EEL_WITNESS_SIGNING_KEY must be exactly 32 bytes"))?;
        Ok(Self::new(
            api_base_url,
            witness_id,
            SigningKey::from_bytes(&seed),
        ))
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

    /// Canonical cosignature message for an STH:
    /// `tree_size` as little-endian u64 followed by the raw root hash bytes.
    pub fn cosign_message(sth: &SignedTreeHead) -> anyhow::Result<Vec<u8>> {
        let root = hex::decode(&sth.root_hash)
            .map_err(|e| anyhow::anyhow!("sth root_hash is not hex: {e}"))?;
        let mut msg = Vec::with_capacity(8 + root.len());
        msg.extend_from_slice(&sth.tree_size.to_le_bytes());
        msg.extend_from_slice(&root);
        Ok(msg)
    }

    /// Sign an STH per the witness protocol and return the hex signature.
    pub fn sign_sth(&self, sth: &SignedTreeHead) -> anyhow::Result<String> {
        let msg = Self::cosign_message(sth)?;
        Ok(hex::encode(self.signing_key.sign(&msg).to_bytes()))
    }

    async fn cosign_latest(&self) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/transparency/sth/latest", self.api_base_url);
        let sth: SignedTreeHead = client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let signature = self.sign_sth(&sth)?;
        let cosign_url = format!("{}/api/v1/transparency/witnesses/cosign", self.api_base_url);
        let body = serde_json::json!({
            "sth_id": sth.id,
            "witness_id": self.witness_id,
            "signature": signature,
        });
        client
            .post(&cosign_url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        info!(tree_size = sth.tree_size, witness_id = %self.witness_id, "cosigned STH");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sth(tree_size: u64, root_hex: &str) -> SignedTreeHead {
        SignedTreeHead {
            id: uuid::Uuid::now_v7(),
            tree_size,
            root_hash: root_hex.into(),
            domain: None,
            cadence: tlog_domain::SthCadence::Default,
            signed_at: chrono::Utc::now(),
            signature: "00".into(),
            key_role: "sth".into(),
            key_fingerprint: "fp".into(),
        }
    }

    #[test]
    fn cosign_message_is_le_size_then_root_bytes() {
        let s = sth(0x0102_0304_0506_0708, "aabb");
        let msg = IabWitnessClient::cosign_message(&s).unwrap();
        assert_eq!(msg.len(), 10);
        assert_eq!(&msg[..8], &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
        assert_eq!(&msg[8..], &[0xaa, 0xbb]);
    }

    #[test]
    fn cosign_message_rejects_non_hex_root() {
        let s = sth(1, "zzzz");
        assert!(IabWitnessClient::cosign_message(&s).is_err());
    }

    #[test]
    fn signature_verifies_under_witness_key() {
        use ed25519_dalek::Verifier;
        let key = SigningKey::from_bytes(&[9u8; 32]);
        let verifying = key.verifying_key();
        let client = IabWitnessClient::new("http://localhost", "iab-1", key);
        let s = sth(42, "deadbeef");
        let sig_hex = client.sign_sth(&s).unwrap();
        let sig = ed25519_dalek::Signature::from_slice(&hex::decode(sig_hex).unwrap()).unwrap();
        let msg = IabWitnessClient::cosign_message(&s).unwrap();
        verifying
            .verify(&msg, &sig)
            .expect("cosignature must verify under the witness key");
    }

    /// End-to-end against an in-process API: the client must fetch the STH,
    /// sign it with the loaded key, and submit the exact cosign contract.
    #[tokio::test]
    async fn cosign_latest_posts_verifiable_signature() {
        use axum::{routing::get, routing::post, Json, Router};
        use ed25519_dalek::Verifier;
        let key = SigningKey::from_bytes(&[5u8; 32]);
        let verifying = key.verifying_key();
        let sth_fixture = sth(7, "00112233");

        let captured: Arc<parking_lot::Mutex<Option<serde_json::Value>>> = Arc::default();
        let cap_route = captured.clone();
        let fixture = Arc::new(sth_fixture);

        let app = Router::new()
            .route(
                "/api/v1/transparency/sth/latest",
                get(move || {
                    let f = fixture.clone();
                    async move { Json((*f).clone()) }
                }),
            )
            .route(
                "/api/v1/transparency/witnesses/cosign",
                post(move |Json(body): Json<serde_json::Value>| {
                    let cap = cap_route.clone();
                    async move {
                        *cap.lock() = Some(body);
                        axum::http::StatusCode::OK
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let client = Arc::new(IabWitnessClient::new(
            format!("http://{addr}"),
            "iab-test",
            key,
        ));
        client.cosign_latest().await.expect("cosign round trip");

        let posted = captured.lock().clone().expect("cosign posted");
        assert_eq!(posted["witness_id"], "iab-test");
        let sig_hex = posted["signature"].as_str().unwrap();
        let sig = ed25519_dalek::Signature::from_slice(&hex::decode(sig_hex).unwrap()).unwrap();
        verifying
            .verify(
                &IabWitnessClient::cosign_message(&sth(7, "00112233")).unwrap(),
                &sig,
            )
            .expect("posted cosignature must verify");
    }
}
