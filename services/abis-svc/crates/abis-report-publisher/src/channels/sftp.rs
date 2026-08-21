//! Generic SFTP channel adapter using ssh2 (blocking, wrapped in spawn_blocking).
//!
//! Uploads the artifact bytes to the SFTP server, then writes a .sha256 sidecar
//! file for offline verification.
//!
//! Configuration per subscriber:
//! - endpoint: "host:port/remote/path" — parsed as host:port and upload directory.
//! - auth_ref: OpenBao path to a PEM-encoded private key.
//!   Env var EEL_SFTP_PRIVKEY_PATH is used as fallback for tests.

use chrono::Utc;
use ssh2::Session;
use std::io::Write as _;
use std::net::TcpStream;
use tracing::{info, warn};

use crate::domain::{DeliveryContext, DeliveryOutcome};

pub async fn deliver(ctx: DeliveryContext) -> DeliveryOutcome {
	// Resolve artifact bytes from blob_ref (MinIO/S3).
	// In production the blob_ref is a presigned URL; for simplicity we fetch it
	// via reqwest. Channel adapters should be given artifact bytes directly in
	// a future refactor.
	let blob_ref = ctx.artifact.blob_ref.clone();
	let artifact_id = ctx.artifact.artifact_id;
	let sha256_hex = hex::encode(&ctx.artifact.sha256);
	let format = ctx.artifact.format.clone();
	let delivery_id = ctx.delivery.delivery_id;

	let endpoint = ctx.subscriber.endpoint.clone();
	let privkey_path = std::env::var("EEL_SFTP_PRIVKEY_PATH").ok();

	// Parse endpoint: "host:port/remote/dir"
	let (hostport, remote_dir) = match endpoint.split_once('/') {
		Some((hp, dir)) => (hp.to_string(), format!("/{dir}")),
		None => (endpoint.clone(), "/uploads".to_string()),
	};

	// Fetch artifact bytes.
	let bytes = match fetch_bytes(&blob_ref).await {
		Ok(b) => b,
		Err(e) => {
			return DeliveryOutcome::Retryable {
				error: format!("failed to fetch artifact bytes: {e}"),
			}
		}
	};

	let filename = format!("{artifact_id}.{format}");
	let sidecar = format!("{artifact_id}.{format}.sha256");
	let sidecar_content = format!("{sha256_hex}  {filename}\n");

	let result = tokio::task::spawn_blocking(move || {
		upload_via_sftp(
			&hostport,
			&remote_dir,
			&filename,
			&sidecar,
			&bytes,
			sidecar_content.as_bytes(),
			privkey_path.as_deref(),
		)
	})
	.await;

	match result {
		Ok(Ok(remote_path)) => {
			info!(
				%delivery_id,
				%remote_path,
				"sftp upload succeeded"
			);
			DeliveryOutcome::Succeeded {
				remote_receipt_id: Some(remote_path),
				remote_receipt_at: Some(Utc::now()),
			}
		}
		Ok(Err(e)) => {
			warn!(%delivery_id, error = %e, "sftp upload failed");
			DeliveryOutcome::Retryable {
				error: format!("sftp upload failed: {e}"),
			}
		}
		Err(e) => DeliveryOutcome::Retryable {
			error: format!("spawn_blocking panicked: {e}"),
		},
	}
}

async fn fetch_bytes(blob_ref: &str) -> Result<Vec<u8>, String> {
	let client = reqwest::Client::builder()
		.timeout(std::time::Duration::from_secs(60))
		.build()
		.map_err(|e| e.to_string())?;
	let resp = client
		.get(blob_ref)
		.send()
		.await
		.map_err(|e| e.to_string())?;
	let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
	Ok(bytes.to_vec())
}

fn upload_via_sftp(
	hostport: &str,
	remote_dir: &str,
	filename: &str,
	sidecar: &str,
	bytes: &[u8],
	sidecar_bytes: &[u8],
	privkey_path: Option<&str>,
) -> Result<String, String> {
	let tcp = TcpStream::connect(hostport).map_err(|e| format!("tcp connect: {e}"))?;
	let mut sess = Session::new().map_err(|e| format!("ssh session: {e}"))?;
	sess.set_tcp_stream(tcp);
	sess.handshake().map_err(|e| format!("ssh handshake: {e}"))?;

	let username = std::env::var("EEL_SFTP_USER").unwrap_or_else(|_| "eel".into());

	if let Some(key_path) = privkey_path {
		sess.userauth_pubkey_file(&username, None, std::path::Path::new(key_path), None)
			.map_err(|e| format!("pubkey auth: {e}"))?;
	} else {
		return Err("no private key path configured; set EEL_SFTP_PRIVKEY_PATH".into());
	}

	if !sess.authenticated() {
		return Err("SSH authentication failed".into());
	}

	let sftp = sess.sftp().map_err(|e| format!("sftp init: {e}"))?;

	let remote_path = format!("{remote_dir}/{filename}");
	let mut f = sftp
		.create(std::path::Path::new(&remote_path))
		.map_err(|e| format!("sftp create {remote_path}: {e}"))?;
	f.write_all(bytes).map_err(|e| format!("sftp write: {e}"))?;

	let sidecar_path = format!("{remote_dir}/{sidecar}");
	let mut sf = sftp
		.create(std::path::Path::new(&sidecar_path))
		.map_err(|e| format!("sftp create sidecar: {e}"))?;
	sf.write_all(sidecar_bytes)
		.map_err(|e| format!("sftp write sidecar: {e}"))?;

	Ok(remote_path)
}
