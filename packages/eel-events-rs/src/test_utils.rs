use eel_crypto_rs::ed25519::{generate_keypair, Ed25519SigningKey};

use crate::spiffe::SpiffeKeyProvider;

/// Ephemeral Ed25519 key provider for use in unit tests.
///
/// Generates a fresh key pair on construction. Not suitable for production.
pub struct EphemeralKeyProvider {
    key: Ed25519SigningKey,
    key_id: String,
}

impl EphemeralKeyProvider {
    pub fn new() -> Self {
        Self {
            key: generate_keypair(),
            key_id: "test-ephemeral-key-1".to_string(),
        }
    }
}

impl Default for EphemeralKeyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SpiffeKeyProvider for EphemeralKeyProvider {
    fn signing_key(&self) -> &Ed25519SigningKey {
        &self.key
    }

    fn key_id(&self) -> &str {
        &self.key_id
    }
}
