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

//! Gov Printer channel adapter.
//!
//! Posts to the py/report-deliverers gov-printer bridge service and returns
//! the submission_id as the remote receipt.
//!
//! Endpoint: EEL_GOV_PRINTER_ADAPTER_URL (default http://localhost:8091)

use chrono::Utc;
use tracing::{info, warn};

use crate::domain::{DeliveryContext, DeliveryOutcome};

pub async fn deliver(ctx: DeliveryContext) -> DeliveryOutcome {
    let base_url = std::env::var("EEL_GOV_PRINTER_ADAPTER_URL")
        .unwrap_or_else(|_| "http://localhost:8091".into());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("reqwest client build");

    let payload = serde_json::json!({
        "artifact_ref": ctx.artifact.blob_ref,
        "artifact_id": ctx.artifact.artifact_id,
        "subscription_id": ctx.subscription.subscription_id,
        "format": ctx.artifact.format,
        "sha256": hex::encode(&ctx.artifact.sha256),
    });

    let url = format!("{base_url}/deliver/gov-printer");

    match client.post(&url).json(&payload).send().await {
        Ok(r) if r.status().is_success() => {
            let body: serde_json::Value = r.json().await.unwrap_or_else(|_| serde_json::json!({}));
            let submission_id = body
                .get("submission_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            info!(
                delivery_id = %ctx.delivery.delivery_id,
                ?submission_id,
                "gov_printer delivery succeeded"
            );
            DeliveryOutcome::Succeeded {
                remote_receipt_id: submission_id,
                remote_receipt_at: Some(Utc::now()),
            }
        }
        Ok(r) if r.status().as_u16() >= 400 && r.status().as_u16() < 500 => {
            DeliveryOutcome::Permanent {
                error: format!("gov_printer adapter returned HTTP {}", r.status()),
            }
        }
        Ok(r) => {
            warn!(
                delivery_id = %ctx.delivery.delivery_id,
                status = %r.status(),
                "gov_printer adapter 5xx; will retry"
            );
            DeliveryOutcome::Retryable {
                error: format!("gov_printer adapter returned HTTP {}", r.status()),
            }
        }
        Err(e) => {
            warn!(
                delivery_id = %ctx.delivery.delivery_id,
                error = %e,
                "gov_printer adapter request failed"
            );
            DeliveryOutcome::Retryable {
                error: format!("gov_printer request failed: {e}"),
            }
        }
    }
}
