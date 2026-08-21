use abis_attest::attestation::{verify_sev_snp, verify_tdx};
use abis_attest::{attest_batch, verify_attestation, AttestationProfile};

#[test]
fn software_profile_always_succeeds() {
    std::env::set_var("EEL_ABIS_ATTESTATION", "software");
    let result = verify_attestation("enc-1", "worker-1", b"any quote bytes here", "nonce-1");
    assert!(result.is_ok());
    let att = result.unwrap();
    assert_eq!(att.profile, AttestationProfile::Software);
}

#[test]
fn software_profile_hash_deterministic() {
    std::env::set_var("EEL_ABIS_ATTESTATION", "software");
    let bytes = b"stable bytes for hashing";
    let r1 = verify_attestation("enc-1", "w-1", bytes, "n1").unwrap();
    let r2 = verify_attestation("enc-1", "w-1", bytes, "n2").unwrap();
    assert_eq!(r1.attestation_hash, r2.attestation_hash);
}

#[test]
fn tdx_invalid_magic_rejected() {
    let quote = vec![0x00u8, 0x01, 0x02, 0x03];
    let result = verify_tdx("enc-1", "w-1", &quote);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, abis_attest::AttestError::InvalidQuote(_)));
}

#[test]
fn tdx_valid_magic_accepted() {
    let quote = vec![0x04u8, 0x00, 0xAA, 0xBB];
    let result = verify_tdx("enc-1", "w-1", &quote);
    assert!(result.is_ok());
    let att = result.unwrap();
    assert_eq!(att.profile, AttestationProfile::Tdx);
}

#[test]
fn sev_snp_valid() {
    let quote = vec![0x01u8, 0x00, 0x00];
    let result = verify_sev_snp("enc-2", "w-2", &quote);
    assert!(result.is_ok());
    let att = result.unwrap();
    assert_eq!(att.profile, AttestationProfile::SevSnp);
}

#[test]
fn sev_snp_invalid() {
    let quote = vec![0xFFu8, 0x00];
    let result = verify_sev_snp("enc-2", "w-2", &quote);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        abis_attest::AttestError::InvalidQuote(_)
    ));
}

#[test]
fn attest_batch_includes_batch_id() {
    std::env::set_var("EEL_ABIS_ATTESTATION", "software");
    let result = attest_batch("enc-1", "w-1", "batch-xyz", b"quote", "nonce");
    assert!(result.is_ok());
    let att = result.unwrap();
    assert_eq!(att.batch_id, Some("batch-xyz".to_string()));
}

#[test]
fn attestation_replay_negative() {
    std::env::set_var("EEL_ABIS_ATTESTATION", "software");
    let nonce = "same-nonce";
    let r1 = verify_attestation("enc-1", "w-1", b"data", nonce);
    let r2 = verify_attestation("enc-1", "w-1", b"data", nonce);
    assert!(r1.is_ok());
    assert!(r2.is_ok());
}
