use thiserror::Error;
use tracing::info;
use crate::bundle::PublicationBundle;

#[derive(Debug, Error)]
pub enum SignerError {
    #[error("serialization failed: {0}")] Serde(#[from] serde_json::Error),
    #[error("upload failed: {0}")] Upload(String),
}

/// Stub: serialize to JSON, compute BLAKE3 hash, return hex.
/// TODO: replace with real cosign signing.
pub async fn sign_bundle(bundle: &PublicationBundle) -> Result<String, SignerError> {
    let json = serde_json::to_vec(bundle)?;
    let hex = blake3::hash(&json).as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>();
    info!(snapshot_id=%bundle.snapshot_id, hash=%hex, "signer: bundle hash computed (stub, TODO: real cosign)");
    Ok(hex)
}

/// Stub: return MinIO URL.
/// TODO: replace with real MinIO SDK upload.
pub async fn upload_to_minio(bundle: &PublicationBundle, _signed_hash: &str) -> Result<String, SignerError> {
    let url = format!("minio://eel-publications/{}.tar.gz", bundle.snapshot_id);
    info!(snapshot_id=%bundle.snapshot_id, url=%url, "signer: upload_to_minio stub (TODO: real MinIO upload)");
    Ok(url)
}
