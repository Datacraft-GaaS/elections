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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Hardware tier of the kit, determines attestation verification path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KitTier {
    /// KIEMS device with TPM 2.0.
    Tpm,
    /// Android device using Play Integrity API.
    PlayIntegrity,
    /// iOS device using App Attest.
    AppAttest,
    /// Legacy Android without hardware attestation root.
    Legacy,
}

/// Integrity verdict returned after successful attestation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntegrityLevel {
    /// Strong hardware-backed attestation.
    MeetsStrong,
    /// Device-backed but not hardware security module.
    MeetsDevice,
    /// Software attestation only; treated as untrusted.
    MeetsBasic,
}

/// Verdict returned by [`crate::verify_attestation`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationVerdict {
    /// Unique kit device identifier derived from attestation payload.
    pub device_id: String,
    /// Attestation tier of the device.
    pub tier: KitTier,
    /// Timestamp after which this verdict must be re-verified.
    pub expires_at: DateTime<Utc>,
    /// Computed integrity level.
    pub integrity_level: IntegrityLevel,
    /// SHA-256 thumbprint of the kit signing certificate.
    pub kit_cert_thumbprint: String,
    /// Set when tier is Legacy; indicates degraded trust.
    pub legacy_warning: bool,
}
