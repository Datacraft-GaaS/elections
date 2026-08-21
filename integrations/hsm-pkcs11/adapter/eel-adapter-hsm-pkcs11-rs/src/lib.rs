//! HSM PKCS#11 vendor profiles.
//!
//! Provides `HsmProfile::new(vendor: Vendor)` returning a `PkcsPool` with
//! vendor-specific slot/label conventions.
//!
//! Vendors: Thales Luna 7, Utimaco CryptoServer, SoftHSMv2 (dev/test).

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Vendor {
    ThalesLuna7,
    UtimacoOpenusr,
    SoftHsm2,
}

impl Vendor {
    pub fn default_library_path(&self) -> &'static str {
        match self {
            Vendor::ThalesLuna7 => "/usr/lib/libCryptoki2_64.so",
            Vendor::UtimacoOpenusr => "/usr/lib/libcs_pkcs11_R3.so",
            Vendor::SoftHsm2 => "/usr/lib/softhsm/libsofthsm2.so",
        }
    }

    pub fn label_prefix(&self) -> &'static str {
        match self {
            Vendor::ThalesLuna7 => "eel-luna",
            Vendor::UtimacoOpenusr => "eel-utimaco",
            Vendor::SoftHsm2 => "eel-softhsm",
        }
    }

    /// Slot index convention for this vendor.
    pub fn default_slot(&self) -> u64 {
        match self {
            Vendor::ThalesLuna7 => 0,
            Vendor::UtimacoOpenusr => 0,
            Vendor::SoftHsm2 => 0,
        }
    }
}

#[derive(Debug, Error)]
pub enum HsmError {
    #[error("PKCS#11 library not found at {path}")]
    LibraryNotFound { path: String },
    #[error("Slot {slot} not available")]
    SlotUnavailable { slot: u64 },
    #[error("Label {label} not found in slot {slot}")]
    LabelNotFound { label: String, slot: u64 },
}

/// Key roles used by the eel platform.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KeyRole {
    Sth,
    Declaration,
    KitCa,
    CredentialCa,
    CodeSigning,
    CycleKek,
}

impl KeyRole {
    pub fn label_suffix(&self) -> &'static str {
        match self {
            KeyRole::Sth => "sth",
            KeyRole::Declaration => "declaration",
            KeyRole::KitCa => "kit-ca",
            KeyRole::CredentialCa => "credential-ca",
            KeyRole::CodeSigning => "code-signing",
            KeyRole::CycleKek => "cycle-kek",
        }
    }
}

/// Pool configuration returned for a given vendor + key role.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PkcsPool {
    pub vendor: Vendor,
    pub library_path: String,
    pub slot: u64,
    pub label: String,
    pub key_role: KeyRole,
}

/// Build a `PkcsPool` for a vendor and key role using eel naming conventions.
pub struct HsmProfile;

impl HsmProfile {
    pub fn new(vendor: Vendor, role: KeyRole) -> PkcsPool {
        let library_path = vendor.default_library_path().to_string();
        let label = format!("{}-{}", vendor.label_prefix(), role.label_suffix());
        PkcsPool {
            slot: vendor.default_slot(),
            library_path,
            label,
            vendor,
            key_role: role,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn softhsm_sth_label() {
        let pool = HsmProfile::new(Vendor::SoftHsm2, KeyRole::Sth);
        assert_eq!(pool.label, "eel-softhsm-sth");
        assert!(pool.library_path.contains("softhsm"));
    }

    #[test]
    fn luna_declaration_label() {
        let pool = HsmProfile::new(Vendor::ThalesLuna7, KeyRole::Declaration);
        assert_eq!(pool.label, "eel-luna-declaration");
    }

    #[test]
    fn utimaco_cycle_kek() {
        let pool = HsmProfile::new(Vendor::UtimacoOpenusr, KeyRole::CycleKek);
        assert_eq!(pool.label, "eel-utimaco-cycle-kek");
    }

    #[test]
    fn all_roles_have_labels() {
        let roles = [
            KeyRole::Sth,
            KeyRole::Declaration,
            KeyRole::KitCa,
            KeyRole::CredentialCa,
            KeyRole::CodeSigning,
            KeyRole::CycleKek,
        ];
        for role in &roles {
            let pool = HsmProfile::new(Vendor::SoftHsm2, role.clone());
            assert!(!pool.label.is_empty());
        }
    }
}
