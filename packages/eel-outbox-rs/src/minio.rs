//! MinIO blob upload stub.
//!
//! Large payloads (> 1 MiB) must be stored in MinIO before emit().
//! This module provides the upload helper used by callers.

use crate::error::OutboxError;

/// Minimum configuration for MinIO uploads.
#[derive(Debug, Clone)]
pub struct MinioConfig {
    /// Base URL of the MinIO endpoint (e.g. `http://minio.internal:9000`).
    pub endpoint: String,
    /// Bucket name.
    pub bucket: String,
    /// Access key.
    pub access_key: String,
    /// Secret key.
    pub secret_key: String,
}

/// Result of a blob upload.
#[derive(Debug, Clone)]
pub struct BlobRef {
    /// MinIO object key (relative path within bucket).
    pub key: String,
    /// BLAKE3 hex hash of the uploaded bytes.
    pub blake3_hash: String,
    /// Content length in bytes.
    pub size: usize,
}

/// Upload raw bytes to MinIO and return a reference.
///
/// The returned `BlobRef` should be embedded in the event `data` field
/// instead of the raw bytes, so the envelope stays under 1 MiB.
#[cfg(feature = "minio")]
pub async fn upload_blob(
    config: &MinioConfig,
    object_key: &str,
    content_type: &str,
    bytes: &[u8],
) -> Result<BlobRef, OutboxError> {
    use reqwest::Client;

    let url = format!("{}/{}/{}", config.endpoint.trim_end_matches('/'), config.bucket, object_key);
    let hash = blake3_hex(bytes);

    let client = Client::new();
    let response = client
        .put(&url)
        .header("Content-Type", content_type)
        .header("Content-Length", bytes.len().to_string())
        .basic_auth(&config.access_key, Some(&config.secret_key))
        .body(bytes.to_vec())
        .send()
        .await
        .map_err(|e| OutboxError::BlobUpload(e.to_string()))?;

    if !response.status().is_success() {
        return Err(OutboxError::BlobUpload(format!(
            "MinIO PUT {} returned {}",
            url,
            response.status()
        )));
    }

    Ok(BlobRef {
        key: object_key.to_string(),
        blake3_hash: hash,
        size: bytes.len(),
    })
}

fn blake3_hex(bytes: &[u8]) -> String {
    let hash = blake3::hash(bytes);
    hash.to_hex().to_string()
}
