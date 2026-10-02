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

//! Webhook channel adapter.
//!
//! POSTs a JSON payload to the subscriber's endpoint URL with:
//! - X-Eel-Signature: HMAC-SHA256 of the JSON body (secret from OpenBao path in auth_ref)
//! - X-Eel-Delivery-Id: UUID of this delivery attempt
//! - Content-Type: application/json

use chrono::Utc;
use reqwest::Client;
use sha2::Digest;
use tracing::{info, warn};

use crate::domain::{DeliveryContext, DeliveryOutcome};

pub async fn deliver(ctx: DeliveryContext) -> DeliveryOutcome {
	let client = Client::builder()
		.timeout(std::time::Duration::from_secs(30))
		.build()
		.expect("reqwest client build never fails with these options");

	let payload = serde_json::json!({
		"delivery_id": ctx.delivery.delivery_id,
		"run_id": ctx.delivery.run_id,
		"artifact_id": ctx.artifact.artifact_id,
		"format": ctx.artifact.format,
		"sha256": hex::encode(&ctx.artifact.sha256),
		"minio_ref": ctx.artifact.blob_ref,
		"tlog_leaf_index": ctx.tlog_anchor.as_ref().map(|a| a.tlog_leaf_index),
	});

	let body = match serde_json::to_vec(&payload) {
		Ok(b) => b,
		Err(e) => {
			return DeliveryOutcome::Permanent {
				error: format!("payload serialization failed: {e}"),
			}
		}
	};

	// Compute HMAC-SHA256 signature.
	// Secret is the auth_ref value (or empty string if not configured).
	let secret = ctx.subscriber.auth_ref.clone().unwrap_or_default();
	let signature = hmac_sha256(secret.as_bytes(), &body);

	let url = ctx.subscriber.endpoint.clone();

	let resp = client
		.post(&url)
		.header("Content-Type", "application/json")
		.header("X-Eel-Signature", format!("sha256={}", hex::encode(signature)))
		.header("X-Eel-Delivery-Id", ctx.delivery.delivery_id.to_string())
		.body(body)
		.send()
		.await;

	match resp {
		Ok(r) if r.status().is_success() => {
			info!(
				delivery_id = %ctx.delivery.delivery_id,
				status = %r.status(),
				url = %url,
				"webhook delivered"
			);
			let receipt = r.headers()
				.get("X-Request-Id")
				.or_else(|| r.headers().get("X-Trace-Id"))
				.and_then(|v| v.to_str().ok())
				.map(String::from);
			DeliveryOutcome::Succeeded {
				remote_receipt_id: receipt,
				remote_receipt_at: Some(Utc::now()),
			}
		}
		Ok(r) if r.status().as_u16() >= 400 && r.status().as_u16() < 500 => {
			// 4xx are permanent failures (misconfigured endpoint, auth failure, etc.)
			DeliveryOutcome::Permanent {
				error: format!("webhook returned HTTP {}", r.status()),
			}
		}
		Ok(r) => {
			warn!(
				delivery_id = %ctx.delivery.delivery_id,
				status = %r.status(),
				"webhook returned 5xx; will retry"
			);
			DeliveryOutcome::Retryable {
				error: format!("webhook returned HTTP {}", r.status()),
			}
		}
		Err(e) => {
			warn!(delivery_id = %ctx.delivery.delivery_id, error = %e, "webhook request failed");
			DeliveryOutcome::Retryable {
				error: format!("webhook request failed: {e}"),
			}
		}
	}
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
	// HMAC-SHA256: H((key XOR opad) || H((key XOR ipad) || data))
	const BLOCK: usize = 64;
	let mut k = [0u8; BLOCK];
	if key.len() <= BLOCK {
		k[..key.len()].copy_from_slice(key);
	} else {
		let h = sha2::Sha256::digest(key);
		k[..32].copy_from_slice(&h);
	}
	let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
	let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();

	let inner = {
		let mut h = sha2::Sha256::new();
		h.update(&ipad);
		h.update(data);
		h.finalize()
	};
	let outer = {
		let mut h = sha2::Sha256::new();
		h.update(&opad);
		h.update(inner);
		h.finalize()
	};
	outer.to_vec()
}
