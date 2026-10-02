//! The RO desktop HSM trust anchor.
//!
//! Verifying a countersignature needs the public key of the HSM that produced it.
//! Those keys are provisioned per returning officer and are not in this
//! repository — they are issued with the hardware.
//!
//! The store is therefore empty by default, which is the point: an unconfigured
//! deployment refuses to verify rather than reporting `status: "verified"` for a
//! signature it never read.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// RO id (as presented in `VerifyForm34ARequest.ro_id`) to hex-encoded Ed25519
/// verifying key.
#[derive(Default)]
pub struct RoTrust {
    keys: RwLock<HashMap<String, String>>,
}

/// The handle threaded through the router.
pub type SharedRoTrust = Arc<RoTrust>;

impl RoTrust {
    /// An empty trust store.
    ///
    /// Returns the value rather than the handle so a caller can trust a key
    /// before sharing it.
    pub fn new() -> Self {
        Self::default()
    }

    /// The shared handle for a configured trust store.
    pub fn shared(self) -> SharedRoTrust {
        Arc::new(self)
    }

    /// Trust a key for an officer.
    pub fn trust(self, ro_id: impl Into<String>, hex_key: impl Into<String>) -> Self {
        self.keys
            .write()
            .expect("lock poisoned")
            .insert(ro_id.into(), hex_key.into());
        self
    }

    /// The key for an officer, if one is configured.
    pub fn key_for(&self, ro_id: &str) -> Option<String> {
        self.keys.read().expect("lock poisoned").get(ro_id).cloned()
    }
}
