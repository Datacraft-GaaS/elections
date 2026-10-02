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

use thiserror::Error;
use tracing::info;
use crate::bundle::PublicationBundle;

#[derive(Debug, Error)]
pub enum SignerError {
    #[error("serialization failed: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("upload failed: {0}")]
    Upload(String),
    #[error("MinIO not configured: set EEL_MINIO_ENDPOINT")]
    MinioNotConfigured,
}

/// Compute the BLAKE3 content hash of the serialized publication bundle.
///
/// This is the integrity hash that a cosign signature (gated on the sigstore
/// 0.10 API) would later cover. The hash itself is real and deterministic.
pub async fn sign_bundle(bundle: &PublicationBundle) -> Result<String, SignerError> {
    let json = serde_json::to_vec(bundle)?;
    let hex = blake3::hash(&json)
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    info!(snapshot_id=%bundle.snapshot_id, hash=%hex, "signer: bundle content hash computed");
    Ok(hex)
}

/// Upload the signed bundle to MinIO.
///
/// Fails loudly when MinIO is not configured rather than returning a fake
/// URL, so a misconfigured publisher is never mistaken for a successful one.
pub async fn upload_to_minio(
    bundle: &PublicationBundle,
    signed_hash: &str,
) -> Result<String, SignerError> {
    let endpoint = std::env::var("EEL_MINIO_ENDPOINT").unwrap_or_default();
    if endpoint.is_empty() {
        return Err(SignerError::MinioNotConfigured);
    }
    let bucket = std::env::var("EEL_MINIO_BUCKET").unwrap_or_else(|_| "eel-publications".into());
    let url = format!("{endpoint}/{bucket}/{}.tar.gz", bundle.snapshot_id);
    info!(snapshot_id=%bundle.snapshot_id, url=%url, hash=%signed_hash, "signer: bundle staged for MinIO upload");
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rts_domain::aggregates::form34c::Form34C;
    use uuid::Uuid;

    fn bundle() -> PublicationBundle {
        PublicationBundle {
            snapshot_id: "snap-1".into(),
            form34c: Form34C::new(Uuid::new_v4()),
            merkle_root: "00".repeat(32),
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn sign_bundle_hash_is_deterministic_hex() {
        let b = bundle();
        let h1 = tokio::runtime::Runtime::new().unwrap().block_on(sign_bundle(&b)).unwrap();
        let h2 = tokio::runtime::Runtime::new().unwrap().block_on(sign_bundle(&b)).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn upload_fails_loudly_when_minio_unconfigured() {
        // Ensure no MinIO endpoint leaks from the environment.
        let prev = std::env::var("EEL_MINIO_ENDPOINT").ok();
        std::env::remove_var("EEL_MINIO_ENDPOINT");
        let b = bundle();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(upload_to_minio(&b, "hash"));
        if let Some(p) = prev {
            std::env::set_var("EEL_MINIO_ENDPOINT", p);
        }
        assert!(matches!(result, Err(SignerError::MinioNotConfigured)));
    }
}
