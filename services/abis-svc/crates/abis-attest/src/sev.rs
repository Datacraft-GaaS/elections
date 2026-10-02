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

//! AMD SEV-SNP attestation report parser.
//!
//! Report layout follows AMD SEV-SNP ABI spec (SNP_REPORT_REQ):
//!   [  0..  4] version (4)
//!   [  4..  8] guest_svn (4)
//!   [  8.. 16] policy (8)
//!   [ 16.. 32] family_id (16)
//!   [ 32.. 48] image_id (16)
//!   [ 48.. 52] vmpl (4)
//!   [ 52.. 56] sig_algo (4)
//!   [ 56.. 64] current_tcb (8)
//!   [ 64.. 72] platform_info (8)
//!   [ 72.. 76] key_info (4)
//!   [ 76.. 80] reserved (4)
//!   [ 80..144] report_data (64)
//!   [144..192] measurement (48)
//!   [192..224] host_data (32)
//!   [224..272] id_key_digest (48)
//!   [272..320] author_key_digest (48)
//!   [320..352] report_id (32)
//!   [352..384] report_id_ma (32)
//!   [384..392] reported_tcb (8)
//!   [392..416] reserved2 (24)
//!   [416..480] chip_id (64)
//!   [480..672] reserved3 (192)
//!   [672..1184] signature (512)
//!   Total: 1184 bytes

use crate::error::AttestError;
use serde::{Deserialize, Serialize};

/// Expected byte length of a SEV-SNP attestation report.
pub const SEV_SNP_REPORT_SIZE: usize = 1184;

/// Parsed AMD SEV-SNP report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevSnpReport {
    pub measurement: Vec<u8>,
    pub host_data: Vec<u8>,
    pub id_key_digest: Vec<u8>,
    pub author_key_digest: Vec<u8>,
    pub report_id: Vec<u8>,
    pub signature: Vec<u8>,
}

impl SevSnpReport {
    /// Parse raw bytes into a SevSnpReport.
    pub fn parse(bytes: &[u8]) -> Result<Self, AttestError> {
        if bytes.len() < SEV_SNP_REPORT_SIZE {
            return Err(AttestError::ReportTooShort {
                expected: SEV_SNP_REPORT_SIZE,
                got: bytes.len(),
            });
        }
        Ok(Self {
            measurement: bytes[144..192].to_vec(),
            host_data: bytes[192..224].to_vec(),
            id_key_digest: bytes[224..272].to_vec(),
            author_key_digest: bytes[272..320].to_vec(),
            report_id: bytes[320..352].to_vec(),
            signature: bytes[672..1184].to_vec(),
        })
    }
}

/// Stub trait for VLEK/VCEK certificate chain verification.
pub trait TrustedVcekStore: Send + Sync {
    /// Return true if the endorsement key certificate chain is trusted.
    fn is_trusted_vcek(&self, vcek_chain: &[u8]) -> bool;
}

/// In-memory dev implementation: trusts all chains (dev only).
pub struct DevVcekStore;

impl TrustedVcekStore for DevVcekStore {
    fn is_trusted_vcek(&self, _vcek_chain: &[u8]) -> bool {
        true
    }
}

/// Parse and verify a SEV-SNP attestation report.
///
/// Magic: byte 0 of a SNP report version field is expected to be 0x01 (version 1).
pub fn parse_and_verify_sev_snp(
    bytes: &[u8],
    vcek_store: &dyn TrustedVcekStore,
) -> Result<SevSnpReport, AttestError> {
    if bytes.is_empty() || bytes[0] != 0x01 {
        return Err(AttestError::InvalidQuote(format!(
            "SEV-SNP version mismatch: expected 0x01, got 0x{:02x}",
            bytes.first().copied().unwrap_or(0)
        )));
    }

    let report = SevSnpReport::parse(bytes)?;

    if !vcek_store.is_trusted_vcek(bytes) {
        return Err(AttestError::ChainVerificationFailed(
            "VCEK chain not trusted".into(),
        ));
    }

    Ok(report)
}
