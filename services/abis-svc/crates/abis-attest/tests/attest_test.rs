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

use abis_attest::attestation::{verify_sev_snp, verify_tdx};
use abis_attest::sev::SEV_SNP_REPORT_SIZE;
use abis_attest::tdx::TD_REPORT_SIZE;
use abis_attest::{
    attest_batch, verify_attestation, AttestationPolicy, AttestationProfile, AttestationVerifier,
    NonceStore, ReportPayload, SevSnpReport, TdReport,
};

// ---------------------------------------------------------------------------
// Existing tests (preserved)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// TDX report golden fixture parsing
// ---------------------------------------------------------------------------

fn make_tdx_golden() -> [u8; TD_REPORT_SIZE] {
    let mut buf = [0u8; TD_REPORT_SIZE];
    // Magic byte required by attestation.rs verify_tdx
    buf[0] = 0x04;
    // MRTD: fill bytes 0..48 with 0xAB
    for b in buf[0..48].iter_mut() {
        *b = 0xAB;
    }
    // magic is now 0xAB, not 0x04 - fix: set byte 0 back
    buf[0] = 0x04;
    // cpu_svn at offset 304..306
    let svn: u16 = 7;
    buf[304..306].copy_from_slice(&svn.to_le_bytes());
    buf
}

#[test]
fn tdx_report_parse_golden() {
    let buf = make_tdx_golden();
    let report = TdReport::parse(&buf).expect("parse TdReport");
    // MRTD[0] was overwritten with 0x04 (magic)
    assert_eq!(report.mrtd[0], 0x04);
    assert_eq!(report.cpu_svn, 7);
}

#[test]
fn tdx_report_too_short_rejected() {
    let short = vec![0x04u8; 100];
    let err = TdReport::parse(&short).unwrap_err();
    assert!(matches!(
        err,
        abis_attest::AttestError::ReportTooShort { .. }
    ));
}

// ---------------------------------------------------------------------------
// SEV-SNP report golden fixture parsing
// ---------------------------------------------------------------------------

fn make_sev_snp_golden() -> [u8; SEV_SNP_REPORT_SIZE] {
    let mut buf = [0u8; SEV_SNP_REPORT_SIZE];
    // version field byte 0 = 0x01
    buf[0] = 0x01;
    // measurement at [144..192] = 0xCC
    for b in buf[144..192].iter_mut() {
        *b = 0xCC;
    }
    // report_id at [320..352] = 0xDD
    for b in buf[320..352].iter_mut() {
        *b = 0xDD;
    }
    buf
}

#[test]
fn sev_snp_report_parse_golden() {
    let buf = make_sev_snp_golden();
    let report = SevSnpReport::parse(&buf).expect("parse SevSnpReport");
    assert_eq!(report.measurement[0], 0xCC);
    assert_eq!(report.report_id[0], 0xDD);
}

#[test]
fn sev_snp_report_too_short_rejected() {
    let short = vec![0x01u8; 100];
    let err = SevSnpReport::parse(&short).unwrap_err();
    assert!(matches!(
        err,
        abis_attest::AttestError::ReportTooShort { .. }
    ));
}

// ---------------------------------------------------------------------------
// Nonce freshness and replay detection
// ---------------------------------------------------------------------------

#[test]
fn nonce_fresh_on_issue() {
    let store = NonceStore::new();
    let nonce = store.issue();
    assert!(nonce.is_fresh());
}

#[test]
fn nonce_consume_once_ok() {
    let store = NonceStore::new();
    let nonce = store.issue();
    assert!(store.consume(&nonce.hex()).is_ok());
}

#[test]
fn nonce_replay_rejected() {
    let store = NonceStore::new();
    let nonce = store.issue();
    store.consume(&nonce.hex()).unwrap();
    let err = store.consume(&nonce.hex()).unwrap_err();
    assert!(matches!(err, abis_attest::AttestError::Replay));
}

#[test]
fn nonce_unknown_rejected() {
    let store = NonceStore::new();
    let err = store.consume("deadbeef00112233").unwrap_err();
    assert!(matches!(err, abis_attest::AttestError::NonceNotFound));
}

// ---------------------------------------------------------------------------
// Policy enforcement matrix
// ---------------------------------------------------------------------------

#[test]
fn policy_require_tdx_got_sev_rejected() {
    let buf = make_sev_snp_golden();
    let report = SevSnpReport::parse(&buf).unwrap();
    let policy = AttestationPolicy {
        require_tdx: true,
        require_sev: false,
        allowed_mrtd_hashes: vec![],
        allowed_sev_measurements: vec![],
        min_cpu_svn: 0,
        allow_software_attestation: false,
    };
    let payload = ReportPayload::SevSnp(report);
    let err = policy.enforce(&payload).unwrap_err();
    assert!(matches!(err, abis_attest::AttestError::PolicyViolation(_)));
}

#[test]
fn policy_require_sev_got_tdx_rejected() {
    let buf = make_tdx_golden();
    let report = TdReport::parse(&buf).unwrap();
    let policy = AttestationPolicy {
        require_tdx: false,
        require_sev: true,
        allowed_mrtd_hashes: vec![],
        allowed_sev_measurements: vec![],
        min_cpu_svn: 0,
        allow_software_attestation: false,
    };
    let payload = ReportPayload::Tdx(report);
    let err = policy.enforce(&payload).unwrap_err();
    assert!(matches!(err, abis_attest::AttestError::PolicyViolation(_)));
}

#[test]
fn policy_software_rejected_in_prod() {
    let policy = AttestationPolicy::production();
    let payload = ReportPayload::Software;
    let err = policy.enforce(&payload).unwrap_err();
    assert!(matches!(
        err,
        abis_attest::AttestError::SoftwareAttestationRejected
    ));
}

#[test]
fn policy_software_accepted_in_dev() {
    let policy = AttestationPolicy::development();
    let payload = ReportPayload::Software;
    let verdict = policy.enforce(&payload).unwrap();
    assert!(verdict.approved);
}

#[test]
fn policy_min_cpu_svn_enforced() {
    let mut buf = make_tdx_golden();
    // Set cpu_svn = 2 (below minimum of 5)
    buf[304] = 2;
    buf[305] = 0;
    let report = TdReport::parse(&buf).unwrap();
    let policy = AttestationPolicy {
        require_tdx: false,
        require_sev: false,
        allowed_mrtd_hashes: vec![],
        allowed_sev_measurements: vec![],
        min_cpu_svn: 5,
        allow_software_attestation: false,
    };
    let payload = ReportPayload::Tdx(report);
    let err = policy.enforce(&payload).unwrap_err();
    assert!(matches!(err, abis_attest::AttestError::PolicyViolation(_)));
}

// ---------------------------------------------------------------------------
// Unified AttestationVerifier
// ---------------------------------------------------------------------------

#[test]
fn verifier_software_dev_accepted() {
    let policy = AttestationPolicy::development();
    let verifier = AttestationVerifier::new_dev(policy);
    let nonce = verifier.issue_nonce();
    // Any bytes not matching TDX (0x04) or SEV (0x01) -> Software path
    let result = verifier.verify("enc-dev", b"softquote", &nonce);
    assert!(result.is_ok());
    assert!(result.unwrap().approved);
}

#[test]
fn verifier_replay_rejected_by_nonce() {
    let policy = AttestationPolicy::development();
    let verifier = AttestationVerifier::new_dev(policy);
    let nonce = verifier.issue_nonce();
    let _ = verifier.verify("enc-dev", b"softquote", &nonce);
    let err = verifier
        .verify("enc-dev", b"softquote", &nonce)
        .unwrap_err();
    assert!(matches!(err, abis_attest::AttestError::Replay));
}
