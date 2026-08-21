//! KBC SFTP channel adapter.
//!
//! Posts to the py/report-deliverers kbc-sftp bridge service and returns
//! the remote_path as receipt.
//!
//! Endpoint: EEL_KBC_SFTP_ADAPTER_URL (default http://localhost:8091)

use chrono::Utc;
use tracing::{info, warn};

use crate::domain::{DeliveryContext, DeliveryOutcome};

pub async fn deliver(ctx: DeliveryContext) -> DeliveryOutcome {
	let base_url = std::env::var("EEL_KBC_SFTP_ADAPTER_URL")
		.unwrap_or_else(|_| "http://localhost:8091".into());

	let client = reqwest::Client::builder()
		.timeout(std::time::Duration::from_secs(120))
		.build()
		.expect("reqwest client build");

	let payload = serde_json::json!({
		"artifact_ref": ctx.artifact.blob_ref,
		"artifact_id": ctx.artifact.artifact_id,
		"subscription_id": ctx.subscription.subscription_id,
		"format": ctx.artifact.format,
		"sha256": hex::encode(&ctx.artifact.sha256),
	});

	let url = format!("{base_url}/deliver/kbc-sftp");

	match client.post(&url).json(&payload).send().await {
		Ok(r) if r.status().is_success() => {
			let body: serde_json::Value = r
				.json()
				.await
				.unwrap_or_else(|_| serde_json::json!({}));
			let remote_path = body
				.get("remote_path")
				.and_then(|v| v.as_str())
				.map(String::from);
			info!(
				delivery_id = %ctx.delivery.delivery_id,
				?remote_path,
				"kbc_sftp delivery succeeded"
			);
			DeliveryOutcome::Succeeded {
				remote_receipt_id: remote_path,
				remote_receipt_at: Some(Utc::now()),
			}
		}
		Ok(r) if r.status().as_u16() >= 400 && r.status().as_u16() < 500 => {
			DeliveryOutcome::Permanent {
				error: format!("kbc_sftp adapter returned HTTP {}", r.status()),
			}
		}
		Ok(r) => {
			warn!(
				delivery_id = %ctx.delivery.delivery_id,
				status = %r.status(),
				"kbc_sftp adapter 5xx; will retry"
			);
			DeliveryOutcome::Retryable {
				error: format!("kbc_sftp adapter returned HTTP {}", r.status()),
			}
		}
		Err(e) => {
			warn!(
				delivery_id = %ctx.delivery.delivery_id,
				error = %e,
				"kbc_sftp adapter request failed"
			);
			DeliveryOutcome::Retryable {
				error: format!("kbc_sftp request failed: {e}"),
			}
		}
	}
}
