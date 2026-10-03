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

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{ReportError, Result};

const TLOG_URL_ENV: &str = "TLOG_API_URL";
const POLL_INTERVAL: Duration = Duration::from_secs(3);
const POLL_MAX: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize)]
struct TlogAddLeafRequest {
    sha256: String, // hex
    description: String,
}

#[derive(Debug, Deserialize)]
struct TlogAddLeafResponse {
    leaf_index: i64,
    entry_id: String,
}

#[derive(Debug, Deserialize)]
struct TlogInclusionResponse {
    leaf_index: i64,
    root_hash: String,  // hex
    proof: Vec<String>, // hex-encoded hashes
    tree_size: i64,
    witness_cosigs: Option<serde_json::Value>,
}

/// Anchor a report artifact SHA-256 in the transparency log.
/// Returns (leaf_index, root_hash, proof_bytes, sth_size, witness_cosigs).
pub async fn anchor_artifact(
    client: &reqwest::Client,
    artifact_id: Uuid,
    artifact_sha256: &[u8],
) -> Result<(i64, Vec<u8>, Vec<u8>, i64, serde_json::Value)> {
    let base_url = std::env::var(TLOG_URL_ENV).unwrap_or_else(|_| "http://localhost:9090".into());

    // 1. Submit leaf
    let add_url = format!("{base_url}/v1/add");
    let req = TlogAddLeafRequest {
        sha256: hex::encode(artifact_sha256),
        description: format!("abis-report:{artifact_id}"),
    };
    let add_resp: TlogAddLeafResponse = client
        .post(&add_url)
        .json(&req)
        .send()
        .await
        .map_err(|e| ReportError::TlogAnchor(e.to_string()))?
        .error_for_status()
        .map_err(|e| ReportError::TlogAnchor(e.to_string()))?
        .json()
        .await
        .map_err(|e| ReportError::TlogAnchor(e.to_string()))?;

    info!(
        leaf_index = add_resp.leaf_index,
        entry_id = %add_resp.entry_id,
        "tlog leaf submitted"
    );

    // 2. Poll for inclusion proof with exponential backoff
    let proof_url = format!("{base_url}/v1/proof/{}", add_resp.leaf_index);
    let mut elapsed = Duration::ZERO;
    let mut wait = Duration::from_secs(1);

    loop {
        tokio::time::sleep(wait).await;
        elapsed += wait;

        debug!(
            elapsed_ms = elapsed.as_millis(),
            "polling tlog inclusion proof"
        );

        match client.get(&proof_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let inclusion: TlogInclusionResponse = resp
                    .json()
                    .await
                    .map_err(|e| ReportError::TlogAnchor(e.to_string()))?;

                let root_hash = hex::decode(&inclusion.root_hash)
                    .map_err(|e| ReportError::TlogAnchor(format!("bad root_hash hex: {e}")))?;

                // Serialise proof as concatenated hashes
                let mut proof_bytes = Vec::new();
                for h in &inclusion.proof {
                    let hb = hex::decode(h)
                        .map_err(|e| ReportError::TlogAnchor(format!("bad proof hash hex: {e}")))?;
                    proof_bytes.extend_from_slice(&hb);
                }

                let cosigs = inclusion.witness_cosigs.unwrap_or(serde_json::json!([]));
                return Ok((
                    inclusion.leaf_index,
                    root_hash,
                    proof_bytes,
                    inclusion.tree_size,
                    cosigs,
                ));
            }
            Ok(resp) if resp.status().as_u16() == 404 => {
                // Not yet included
            }
            Ok(resp) => {
                warn!(status = %resp.status(), "unexpected tlog poll response");
            }
            Err(e) => {
                warn!(error = %e, "tlog poll request failed");
            }
        }

        if elapsed >= POLL_MAX {
            return Err(ReportError::TlogAnchor(format!(
                "inclusion proof not available after {POLL_MAX:?}"
            )));
        }

        // Exponential backoff, cap at 10s
        wait = (wait * 2).min(Duration::from_secs(10));
    }
}
