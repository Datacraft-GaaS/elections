use eel_crypto_rs::ed25519::Ed25519SigningKey;

/// Provides the active SPIFFE-issued Ed25519 signing key for event envelopes.
///
/// Implementations are expected to rotate the key hourly and update
/// `key_id` accordingly. The `EventBuilder::build` method calls through
/// this trait so service code never touches key material directly.
pub trait SpiffeKeyProvider: Send + Sync {
    /// Returns a reference to the current Ed25519 signing key.
    fn signing_key(&self) -> &Ed25519SigningKey;

    /// Returns the SPIFFE SVID fingerprint that identifies this key.
    fn key_id(&self) -> &str;
}
