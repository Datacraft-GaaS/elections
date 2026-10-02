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

use kit_attest::{
    verify_attestation, AttestError, AttestationKind, AttestationPayload, IntegrityLevel, KitTier,
};

#[test]
fn tpm_quote_meets_device() {
    let p = AttestationPayload {
        kind: AttestationKind::TpmQuote,
        token: "tpm-quote-blob".into(),
        device_serial: "KIEMS-TPM-00042".into(),
        nonce: Some("nonce-xyz".into()),
    };
    let v = verify_attestation(&p).unwrap();
    assert_eq!(v.tier, KitTier::Tpm);
    assert_eq!(v.integrity_level, IntegrityLevel::MeetsDevice);
    assert!(!v.legacy_warning);
}

#[test]
fn tampered_tpm_rejected() {
    let p = AttestationPayload {
        kind: AttestationKind::TpmQuote,
        token: "TAMPERED:bad-quote".into(),
        device_serial: "KIEMS-TPM-00042".into(),
        nonce: None,
    };
    assert!(matches!(
        verify_attestation(&p),
        Err(AttestError::Tampered(_))
    ));
}

#[test]
fn verdict_has_expiry_in_future() {
    let p = AttestationPayload {
        kind: AttestationKind::PlayIntegrity,
        token: "valid-token".into(),
        device_serial: "BVRD-KE-00100".into(),
        nonce: None,
    };
    let v = verify_attestation(&p).unwrap();
    assert!(v.expires_at > chrono::Utc::now());
}
