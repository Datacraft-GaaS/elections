//! Ed25519 sign and verify (RFC 8032) using ed25519-dalek.
//!
//! # Example
//!
//! ```
//! use eel_crypto_rs::ed25519::{generate_keypair, sign, verify};
//! let kp = generate_keypair();
//! let msg = b"election data";
//! let sig = sign(&kp, msg);
//! assert!(verify(&kp.verifying_key(), msg, &sig).is_ok());
//! ```

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::{OsRng, RngCore};

use crate::CryptoError;

pub use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey};

/// Generate a fresh Ed25519 key pair using OS randomness.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::ed25519::generate_keypair;
/// let kp = generate_keypair();
/// assert_eq!(kp.verifying_key().as_bytes().len(), 32);
/// ```
pub fn generate_keypair() -> SigningKey {
    // Generate 32 random bytes then construct the key directly.
    // This avoids requiring the `rand_core` feature on ed25519-dalek.
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    SigningKey::from_bytes(&secret)
}

/// Returns the 32-byte public key for `signing_key`.
///
/// Exposed so callers that only need the public key do not have to depend on
/// `ed25519-dalek` directly just to read `verifying_key()`.
pub fn public_key(signing_key: &SigningKey) -> [u8; 32] {
    signing_key.verifying_key().to_bytes()
}

/// Sign `message` with `signing_key`, returning a 64-byte signature.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::ed25519::{generate_keypair, sign};
/// let kp = generate_keypair();
/// let sig = sign(&kp, b"data");
/// assert_eq!(sig.to_bytes().len(), 64);
/// ```
pub fn sign(signing_key: &SigningKey, message: &[u8]) -> Signature {
    signing_key.sign(message)
}

/// Verify `signature` over `message` with `verifying_key`.
///
/// Returns `Ok(())` on success, `Err(CryptoError::InvalidSignature)` on
/// failure.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::ed25519::{generate_keypair, sign, verify};
/// let kp = generate_keypair();
/// let sig = sign(&kp, b"data");
/// assert!(verify(&kp.verifying_key(), b"data", &sig).is_ok());
/// assert!(verify(&kp.verifying_key(), b"wrong", &sig).is_err());
/// ```
pub fn verify(
    verifying_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> Result<(), CryptoError> {
    verifying_key
        .verify(message, signature)
        .map_err(|_| CryptoError::InvalidSignature)
}

/// Decode a 32-byte verifying key from its canonical byte form.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::ed25519::{generate_keypair, verifying_key_from_bytes};
/// let kp = generate_keypair();
/// let bytes = kp.verifying_key().to_bytes();
/// let vk = verifying_key_from_bytes(&bytes).unwrap();
/// assert_eq!(vk, kp.verifying_key());
/// ```
pub fn verifying_key_from_bytes(bytes: &[u8; 32]) -> Result<VerifyingKey, CryptoError> {
    VerifyingKey::from_bytes(bytes).map_err(|e| CryptoError::InvalidKey(e.to_string()))
}

/// Decode a signature from its 64-byte canonical form.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::ed25519::{generate_keypair, sign, signature_from_bytes};
/// let kp = generate_keypair();
/// let sig = sign(&kp, b"msg");
/// let bytes = sig.to_bytes();
/// let back = signature_from_bytes(&bytes).unwrap();
/// assert_eq!(back.to_bytes(), bytes);
/// ```
pub fn signature_from_bytes(bytes: &[u8; 64]) -> Result<Signature, CryptoError> {
    Ok(Signature::from_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn sign_verify_roundtrip() {
        let kp = generate_keypair();
        let msg = b"hello eel";
        let sig = sign(&kp, msg);
        verify(&kp.verifying_key(), msg, &sig).unwrap();
    }

    #[test]
    fn verify_rejects_tampered_message() {
        let kp = generate_keypair();
        let sig = sign(&kp, b"original");
        assert!(verify(&kp.verifying_key(), b"tampered", &sig).is_err());
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let kp1 = generate_keypair();
        let kp2 = generate_keypair();
        let sig = sign(&kp1, b"data");
        assert!(verify(&kp2.verifying_key(), b"data", &sig).is_err());
    }

    proptest! {
        /// Signing with a fixed key and verifying with the same key always succeeds.
        #[test]
        fn prop_sign_verify_any_message(msg in proptest::collection::vec(0u8..=255u8, 0..1024)) {
            let kp = generate_keypair();
            let sig = sign(&kp, &msg);
            prop_assert!(verify(&kp.verifying_key(), &msg, &sig).is_ok());
        }

        /// A signature over one message does not verify over a different message.
        #[test]
        fn prop_sig_does_not_cross_verify(
            msg1 in proptest::collection::vec(0u8..=255u8, 1..64),
            msg2 in proptest::collection::vec(0u8..=255u8, 1..64),
        ) {
            if msg1 != msg2 {
                let kp = generate_keypair();
                let sig = sign(&kp, &msg1);
                prop_assert!(verify(&kp.verifying_key(), &msg2, &sig).is_err());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Kani harness: Ed25519 verify with malformed inputs must not panic.
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod kani_harnesses {
    use super::*;

    /// Verifying with a random 64-byte "signature" must never panic;
    /// it may succeed or fail, but must not panic or abort.
    #[kani::proof]
    fn ed25519_verify_malformed_never_panics() {
        let key_bytes: [u8; 32] = kani::any();
        let sig_bytes: [u8; 64] = kani::any();
        let msg_len: usize = kani::any_where(|n: &usize| *n < 64);
        let mut msg = vec![0u8; msg_len];
        for b in &mut msg {
            *b = kani::any();
        }

        if let Ok(vk) = verifying_key_from_bytes(&key_bytes) {
            let sig = signature_from_bytes(&sig_bytes).unwrap();
            let _ = verify(&vk, &msg, &sig);
        }
    }
}
