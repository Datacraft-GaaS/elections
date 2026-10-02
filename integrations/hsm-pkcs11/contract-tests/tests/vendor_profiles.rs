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
