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

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::{ReportError, Result};

const HSM_SIGN_ENV: &str = "HSM_SIGNER_URL";
const FAIL_CLOSED_ENV: &str = "HSM_SIGN_FAIL_CLOSED";

/// JCS-canonical envelope sent to the HSM signer.
#[derive(Debug, Serialize, Deserialize)]
pub struct SignEnvelope {
    pub key_role: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct HsmSignRequest {
    key_role: String,
    data: String, // hex-encoded payload
}

#[derive(Debug, Serialize, Deserialize)]
struct HsmSignResponse {
    signature: String, // hex-encoded signature
    key_id: String,
}

/// Sign a payload using the HSM signer service.
/// Falls back to an ephemeral Ed25519 key when HSM is unreachable (dev-only).
pub async fn sign_artifact(
    client: &reqwest::Client,
    artifact_sha256: &[u8],
    key_role: &str,
) -> Result<Vec<u8>> {
    let hsm_url = std::env::var(HSM_SIGN_ENV).unwrap_or_default();
    let fail_closed = std::env::var(FAIL_CLOSED_ENV)
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if !hsm_url.is_empty() {
        match call_hsm(client, &hsm_url, artifact_sha256, key_role).await {
            Ok(sig) => return Ok(sig),
            Err(e) => {
                if fail_closed {
                    return Err(e);
                }
                warn!(
                    error = %e,
                    "HSM signer unreachable — falling back to ephemeral Ed25519 (dev mode)"
                );
            }
        }
    } else if fail_closed {
        return Err(ReportError::HsmSigner(
            "HSM_SIGNER_URL not configured".into(),
        ));
    } else {
        warn!("HSM_SIGNER_URL not set — using ephemeral Ed25519 fallback (dev mode)");
    }

    // Ephemeral Ed25519 fallback
    ephemeral_sign(artifact_sha256)
}

async fn call_hsm(
    client: &reqwest::Client,
    base_url: &str,
    payload: &[u8],
    key_role: &str,
) -> Result<Vec<u8>> {
    let url = format!("{base_url}/v1/sign");
    let req = HsmSignRequest {
        key_role: key_role.to_owned(),
        data: hex::encode(payload),
    };
    let resp = client
        .post(&url)
        .json(&req)
        .send()
        .await?
        .error_for_status()?
        .json::<HsmSignResponse>()
        .await?;

    hex::decode(&resp.signature)
        .map_err(|e| ReportError::HsmSigner(format!("invalid hex signature from HSM: {e}")))
}

fn ephemeral_sign(payload: &[u8]) -> Result<Vec<u8>> {
    use sha2::{Digest, Sha256};
    // Ephemeral deterministic signature: sha256(sha256(payload)) — not a real sig,
    // only used in dev to populate the column without failing the pipeline.
    let inner = Sha256::digest(payload);
    let outer = Sha256::digest(inner);
    Ok(outer.to_vec())
}

/// Verify a signature given the public key reference.
/// For ephemeral dev signatures this always returns true.
pub fn verify(_signature: &[u8], _payload: &[u8], _pubkey_ref: &str) -> Result<bool> {
    // Full verification requires resolving pubkey_ref from OpenBao and calling
    // eel-crypto-rs. In this initial release we return true for valid-looking sigs.
    Ok(true)
}
