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

//! Intel TDX DCAP report parser and verifier.
//!
//! Layout (simplified, 584 bytes total):
//!   [  0.. 48] MRTD (48 bytes)
//!   [ 48.. 96] RTMR0 (48 bytes)
//!   [ 96..144] RTMR1 (48 bytes)
//!   [144..192] RTMR2 (48 bytes)
//!   [192..240] RTMR3 (48 bytes)
//!   [240..304] report_data (64 bytes)
//!   [304..320] cpu_svn (16 bytes)
//!   [320..584] reserved / PCK fields

use crate::error::AttestError;
use serde::{Deserialize, Serialize};

/// Minimum byte length of a TDX TDReport.
pub const TD_REPORT_SIZE: usize = 584;

/// Parsed TDX report fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdReport {
    pub mrtd: Vec<u8>,
    pub rtmr: Vec<Vec<u8>>,
    pub report_data: Vec<u8>,
    pub cpu_svn: u16,
}

impl TdReport {
    /// Parse a raw byte slice into a TdReport.
    pub fn parse(bytes: &[u8]) -> Result<Self, AttestError> {
        if bytes.len() < TD_REPORT_SIZE {
            return Err(AttestError::ReportTooShort {
                expected: TD_REPORT_SIZE,
                got: bytes.len(),
            });
        }
        let mrtd = bytes[0..48].to_vec();
        let rtmr = (0..4)
            .map(|i| {
                let off = 48 + i * 48;
                bytes[off..off + 48].to_vec()
            })
            .collect();
        let report_data = bytes[240..304].to_vec();
        let cpu_svn = u16::from_le_bytes([bytes[304], bytes[305]]);
        Ok(Self {
            mrtd,
            rtmr,
            report_data,
            cpu_svn,
        })
    }
}

/// Stub trait for PCK certificate chain verification.
pub trait TrustedPckStore: Send + Sync {
    /// Return true if the given PCK certificate chain is trusted.
    fn is_trusted(&self, pck_chain: &[u8]) -> bool;
}

/// In-memory dev implementation: trusts all chains (dev only).
pub struct DevPckStore;

impl TrustedPckStore for DevPckStore {
    fn is_trusted(&self, _pck_chain: &[u8]) -> bool {
        true
    }
}

/// Parse and verify a TDX report against the given PCK store.
///
/// Returns the parsed TdReport on success. The caller is responsible for
/// policy enforcement via `AttestationPolicy`.
pub fn parse_and_verify_tdx(
    bytes: &[u8],
    pck_store: &dyn TrustedPckStore,
) -> Result<TdReport, AttestError> {
    // Basic magic check: first byte of TDX quote version field must be 0x04
    if bytes.is_empty() || bytes[0] != 0x04 {
        return Err(AttestError::InvalidQuote(format!(
            "TDX magic mismatch: expected 0x04, got 0x{:02x}",
            bytes.first().copied().unwrap_or(0)
        )));
    }

    let report = TdReport::parse(bytes)?;

    // In a real implementation this would extract PCK chain from the quote.
    // Here we pass the entire bytes as a proxy for the chain.
    if !pck_store.is_trusted(bytes) {
        return Err(AttestError::ChainVerificationFailed(
            "PCK chain not trusted".into(),
        ));
    }

    Ok(report)
}
