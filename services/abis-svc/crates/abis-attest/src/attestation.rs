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

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttestationProfile {
    Software,
    Tdx,
    SevSnp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveAttestation {
    pub enclave_id: String,
    pub worker_id: String,
    pub profile: AttestationProfile,
    pub attestation_hash: String,
    pub verified_at: String,
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationResult {
    pub success: bool,
    pub attestation: Option<EnclaveAttestation>,
    pub error: Option<String>,
}

/// Verify attestation. For software profile (EEL_ABIS_ATTESTATION=software),
/// always returns success. For TDX/SEV-SNP, validates a minimal quote structure.
pub fn verify_attestation(
    enclave_id: &str,
    worker_id: &str,
    quote_bytes: &[u8],
    _nonce: &str,
) -> Result<EnclaveAttestation, crate::error::AttestError> {
    let profile = detect_profile();
    match profile {
        AttestationProfile::Software => {
            let hash = compute_hash(quote_bytes);
            Ok(EnclaveAttestation {
                enclave_id: enclave_id.to_string(),
                worker_id: worker_id.to_string(),
                profile: AttestationProfile::Software,
                attestation_hash: hash,
                verified_at: chrono::Utc::now().to_rfc3339(),
                batch_id: None,
            })
        }
        AttestationProfile::Tdx => verify_tdx(enclave_id, worker_id, quote_bytes),
        AttestationProfile::SevSnp => verify_sev_snp(enclave_id, worker_id, quote_bytes),
    }
}

pub(crate) fn detect_profile() -> AttestationProfile {
    match std::env::var("EEL_ABIS_ATTESTATION").as_deref() {
        Ok("software") => AttestationProfile::Software,
        Ok("tdx") => AttestationProfile::Tdx,
        Ok("sev-snp") | Ok("sev_snp") => AttestationProfile::SevSnp,
        _ => AttestationProfile::Software,
    }
}

fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Validate a TDX quote: first byte must be 0x04.
pub fn verify_tdx(
    enclave_id: &str,
    worker_id: &str,
    quote_bytes: &[u8],
) -> Result<EnclaveAttestation, crate::error::AttestError> {
    if quote_bytes.len() < 2 {
        return Err(crate::error::AttestError::InvalidQuote(
            "quote too short for TDX".into(),
        ));
    }
    if quote_bytes[0] != 0x04 {
        return Err(crate::error::AttestError::InvalidQuote(format!(
            "TDX quote magic mismatch: expected 0x04, got 0x{:02x}",
            quote_bytes[0]
        )));
    }
    let hash = compute_hash(quote_bytes);
    Ok(EnclaveAttestation {
        enclave_id: enclave_id.to_string(),
        worker_id: worker_id.to_string(),
        profile: AttestationProfile::Tdx,
        attestation_hash: hash,
        verified_at: chrono::Utc::now().to_rfc3339(),
        batch_id: None,
    })
}

/// Validate a SEV-SNP report: first byte must be 0x01.
pub fn verify_sev_snp(
    enclave_id: &str,
    worker_id: &str,
    quote_bytes: &[u8],
) -> Result<EnclaveAttestation, crate::error::AttestError> {
    if quote_bytes.is_empty() {
        return Err(crate::error::AttestError::InvalidQuote(
            "quote empty for SEV-SNP".into(),
        ));
    }
    if quote_bytes[0] != 0x01 {
        return Err(crate::error::AttestError::InvalidQuote(format!(
            "SEV-SNP report type mismatch: expected 0x01, got 0x{:02x}",
            quote_bytes[0]
        )));
    }
    let hash = compute_hash(quote_bytes);
    Ok(EnclaveAttestation {
        enclave_id: enclave_id.to_string(),
        worker_id: worker_id.to_string(),
        profile: AttestationProfile::SevSnp,
        attestation_hash: hash,
        verified_at: chrono::Utc::now().to_rfc3339(),
        batch_id: None,
    })
}

/// Attest a match batch, appending batch_id to the attestation.
pub fn attest_batch(
    enclave_id: &str,
    worker_id: &str,
    batch_id: &str,
    quote_bytes: &[u8],
    nonce: &str,
) -> Result<EnclaveAttestation, crate::error::AttestError> {
    let mut att = verify_attestation(enclave_id, worker_id, quote_bytes, nonce)?;
    att.batch_id = Some(batch_id.to_string());
    Ok(att)
}
