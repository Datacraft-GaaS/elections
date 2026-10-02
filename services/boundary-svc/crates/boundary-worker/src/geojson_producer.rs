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

//! Signed GeoJSON tarball producer.
//!
//! Cadence: every 15 minutes (driven by `run_all_workers`).
//!
//! Process:
//! 1. Query all active boundaries from PostgreSQL.
//! 2. Serialise to GeoJSON FeatureCollection per boundary kind.
//! 3. Gzip compress to a tarball.
//! 4. Compute SHA-256 of the tarball.
//! 5. HSM-countersign via crates/hsm-signer interface (WP-2.2 provides the
//!    server; we call the stub interface here).
//! 6. Push tarball + detached cosign bundle to MinIO bucket `eel-boundary-geojson`.
//! 7. Consumers verify via verifier-rs before use.
//!
//! Security invariant: unsigned GeoJSON tarballs MUST NOT be served. Forged
//! polygon negative test: if `signature` is empty the producer MUST abort.

use anyhow::{bail, Result};
use sha2::{Digest, Sha256};
use tracing::{info, warn};

pub async fn run_once() -> Result<()> {
    info!("GeoJSON tarball producer: no-op stub");
    Ok(())
}

/// Produce a signed tarball bundle.
///
/// # Errors
/// Returns `Err` if `tarball_bytes` is empty or the HSM signer returns an
/// empty signature (forged-polygon negative path).
pub async fn produce_signed_bundle(tarball_bytes: &[u8]) -> Result<SignedBundle> {
    if tarball_bytes.is_empty() {
        bail!("tarball_bytes must not be empty");
    }

    let mut hasher = Sha256::new();
    hasher.update(tarball_bytes);
    let hash = hex::encode(hasher.finalize());

    // Call HSM signer stub (WP-2.2 provides the concrete impl).
    let signature = hsm_sign_stub(tarball_bytes).await?;

    if signature.is_empty() {
        warn!("HSM signer returned empty signature; aborting tarball push");
        bail!("HSM signer returned empty signature; forged-polygon guard triggered");
    }

    Ok(SignedBundle {
        tarball_bytes: tarball_bytes.to_vec(),
        sha256_hex: hash,
        cosign_bundle: signature,
    })
}

#[derive(Debug)]
pub struct SignedBundle {
    pub tarball_bytes: Vec<u8>,
    pub sha256_hex: String,
    pub cosign_bundle: Vec<u8>,
}

/// Stub for the HSM signer interface. In production this calls the Unix
/// socket exposed by crates/hsm-signer (WP-2.2).
async fn hsm_sign_stub(data: &[u8]) -> Result<Vec<u8>> {
    // Stub: return 64 non-empty bytes so the positive path works in dev.
    // The real implementation calls the PKCS#11 HSM countersign RPC.
    let _ = data;
    Ok(vec![0xAB; 64])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_tarball_is_rejected() {
        let result = produce_signed_bundle(&[]).await;
        assert!(result.is_err(), "empty tarball must be rejected");
    }

    #[tokio::test]
    async fn valid_tarball_produces_bundle() {
        let bundle = produce_signed_bundle(b"fake-tarball-content").await.unwrap();
        assert!(!bundle.sha256_hex.is_empty());
        assert!(!bundle.cosign_bundle.is_empty());
    }
}
