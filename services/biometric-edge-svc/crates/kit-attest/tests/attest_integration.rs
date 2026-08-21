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
