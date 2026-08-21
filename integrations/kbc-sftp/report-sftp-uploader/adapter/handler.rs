//! Rust adapter that calls the py/report-deliverers kbc-sftp bridge service.
//!
//! Default endpoint: http://localhost:8091 (EEL_KBC_SFTP_ADAPTER_URL).

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct KbcSftpRequest {
	pub artifact_ref: String,
	pub artifact_id: String,
	pub subscription_id: String,
	pub format: String,
	pub sha256: String,
}

#[derive(Debug, Deserialize)]
pub struct KbcSftpReceipt {
	pub remote_path: Option<String>,
	pub sha256: Option<String>,
	pub uploaded_at: Option<String>,
}

/// POST to the kbc-sftp bridge and return the upload receipt.
pub async fn deliver(req: KbcSftpRequest) -> Result<KbcSftpReceipt, String> {
	let base = std::env::var("EEL_KBC_SFTP_ADAPTER_URL")
		.unwrap_or_else(|_| "http://localhost:8091".into());
	let url = format!("{base}/deliver/kbc-sftp");

	let client = reqwest::Client::builder()
		.timeout(Duration::from_secs(120))
		.build()
		.map_err(|e| e.to_string())?;

	let resp = client
		.post(&url)
		.json(&req)
		.send()
		.await
		.map_err(|e| format!("request failed: {e}"))?;

	if !resp.status().is_success() {
		return Err(format!("kbc-sftp adapter returned HTTP {}", resp.status()));
	}

	resp.json::<KbcSftpReceipt>()
		.await
		.map_err(|e| format!("response decode failed: {e}"))
}
