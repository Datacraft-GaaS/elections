//! Contract tests for HSM PKCS#11 vendor profiles.

use eel_adapter_hsm_pkcs11_rs::{HsmProfile, KeyRole, Vendor};

#[test]
fn softhsm2_all_key_roles() {
    let roles = [
        (KeyRole::Sth, "eel-softhsm-sth"),
        (KeyRole::Declaration, "eel-softhsm-declaration"),
        (KeyRole::KitCa, "eel-softhsm-kit-ca"),
        (KeyRole::CredentialCa, "eel-softhsm-credential-ca"),
        (KeyRole::CodeSigning, "eel-softhsm-code-signing"),
        (KeyRole::CycleKek, "eel-softhsm-cycle-kek"),
    ];
    for (role, expected_label) in roles {
        let pool = HsmProfile::new(Vendor::SoftHsm2, role);
        assert_eq!(pool.label, expected_label, "label mismatch");
        assert!(pool.library_path.contains("softhsm"), "wrong lib path");
    }
}

#[test]
fn thales_luna_library_path() {
    let pool = HsmProfile::new(Vendor::ThalesLuna7, KeyRole::Sth);
    assert!(pool.library_path.contains("Cryptoki2"));
}

#[test]
fn utimaco_library_path() {
    let pool = HsmProfile::new(Vendor::UtimacoOpenusr, KeyRole::Sth);
    assert!(pool.library_path.contains("cs_pkcs11"));
}
