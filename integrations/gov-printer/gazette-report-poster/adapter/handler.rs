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

//! Rust adapter that calls the py/report-deliverers gov-printer bridge service.
//!
//! Default endpoint: http://localhost:8091 (EEL_GOV_PRINTER_ADAPTER_URL).
//!
//! This file is a standalone Rust module; it is NOT part of the workspace
//! (it lives under integrations/, outside the `services/*/crates/*` glob).
//! Include it in a binary crate that provides main() and workspace membership.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct GovPrinterRequest {
	pub artifact_ref: String,
	pub artifact_id: String,
	pub subscription_id: String,
	pub format: String,
	pub sha256: String,
}

#[derive(Debug, Deserialize)]
pub struct GovPrinterReceipt {
	pub submission_id: Option<String>,
	pub submitted_at: Option<String>,
}

/// POST to the gov-printer bridge and return the submission receipt.
pub async fn deliver(req: GovPrinterRequest) -> Result<GovPrinterReceipt, String> {
	let base = std::env::var("EEL_GOV_PRINTER_ADAPTER_URL")
		.unwrap_or_else(|_| "http://localhost:8091".into());
	let url = format!("{base}/deliver/gov-printer");

	let client = reqwest::Client::builder()
		.timeout(Duration::from_secs(60))
		.build()
		.map_err(|e| e.to_string())?;

	let resp = client
		.post(&url)
		.json(&req)
		.send()
		.await
		.map_err(|e| format!("request failed: {e}"))?;

	if !resp.status().is_success() {
		return Err(format!("gov-printer adapter returned HTTP {}", resp.status()));
	}

	resp.json::<GovPrinterReceipt>()
		.await
		.map_err(|e| format!("response decode failed: {e}"))
}
