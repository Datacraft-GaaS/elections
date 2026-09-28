//! Authenticated encryption with associated data, for data at rest.
//!
//! One AEAD: **AES-256-GCM**, via the `ring` crate. AES-GCM is a NIST
//! standard (SP 800-38D) and hardware-accelerated on the x86 and ARM cores in
//! the target hardware, which matters for offline kit encryption throughput.
//! `ring` is already in the dependency graph, so this adds no new supply-chain
//! surface.
//!
//! # Why associated data is not optional
//!
//! Every call here binds a caller-supplied associated-data blob into the
//! authentication tag. Callers **must** include the identity of the thing being
//! protected -- record id, instance id, or similar -- and not merely a constant.
//! Without it, a ciphertext is valid wherever it is pasted: an attacker could
//! lift an encrypted datum from one citizen's record and drop it into another's,
//! or replay a record from one station into another, and decryption would
//! succeed. Binding the identity makes the ciphertext non-transplantable.
//!
//! # Nonce discipline
//!
//! GCM nonces must never repeat under a given key. [`seal`] takes the nonce
//! from the operating system's CSPRNG and generates it fresh per call; it is
//! prepended to the returned blob so it travels with the ciphertext. Callers
//! must not supply nonces themselves, and must not reuse a data key across more
//! [`NONCE_LEN`] messages.

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::error::Unspecified;
use ring::rand::SecureRandom;

use crate::error::CryptoError;

/// Data key length for AES-256.
pub const KEY_LEN: usize = 32;

/// AEAD nonce length for AES-GCM.
pub const NONCE_LEN: usize = ring::aead::NONCE_LEN;

/// GCM authentication tag length.
pub const TAG_LEN: usize = 16;

/// Generate a random 256-bit data key.
pub fn generate_data_key() -> Result<[u8; KEY_LEN], CryptoError> {
    let mut key = [0u8; KEY_LEN];
    ring::rand::SystemRandom::new()
        .fill(&mut key)
        .map_err(|_| CryptoError::Randomness)?;
    Ok(key)
}

/// Encrypt `plaintext` under `key`, binding `aad` into the authentication tag.
///
/// Returns `nonce || ciphertext || tag`. The nonce is generated internally, so
/// callers cannot accidentally reuse one.
pub fn seal(key: &[u8; KEY_LEN], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    ring::rand::SystemRandom::new()
        .fill(&mut nonce_bytes)
        .map_err(|_| CryptoError::Randomness)?;

    let unbound = UnboundKey::new(&AES_256_GCM, key).map_err(|_| CryptoError::Aead)?;
    let sealing = LessSafeKey::new(unbound);

    // seal_in_place_append_tag seals in place and appends the tag, so the
    // plaintext has to start in a mutable buffer and the result is the buffer.
    let mut in_out = plaintext.to_vec();
    sealing
        .seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::from(AadTag { data: aad }),
            &mut in_out,
        )
        .map_err(|_| CryptoError::Aead)?;

    let mut out = Vec::with_capacity(NONCE_LEN + in_out.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&in_out);
    Ok(out)
}

/// Decrypt a blob produced by [`seal`], verifying `aad` matches.
///
/// Fails closed: a wrong key, a modified ciphertext, a modified tag, or
/// mismatched associated data are indistinguishable to the caller and all
/// yield [`CryptoError::Aead`]. That is deliberate -- distinguishing them would
/// leak information to an attacker probing the store.
pub fn open(key: &[u8; KEY_LEN], sealed: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if sealed.len() < NONCE_LEN + TAG_LEN {
        return Err(CryptoError::Aead);
    }
    let unbound = UnboundKey::new(&AES_256_GCM, key).map_err(|_| CryptoError::Aead)?;
    let opening = LessSafeKey::new(unbound);

    let (nonce_bytes, rest) = sealed.split_at(NONCE_LEN);
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(nonce_bytes);
    let mut in_out = rest.to_vec();

    let plaintext = opening
        .open_in_place(
            Nonce::assume_unique_for_key(nonce),
            Aad::from(AadTag { data: aad }),
            &mut in_out,
        )
        .map_err(|_| CryptoError::Aead)?;
    Ok(plaintext.to_vec())
}

/// Wrapper so the associated-data slice carries a length, as `ring` requires.
struct AadTag<'a> {
    data: &'a [u8],
}

impl AsRef<[u8]> for AadTag<'_> {
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl From<Unspecified> for CryptoError {
    fn from(_: Unspecified) -> Self {
        CryptoError::Aead
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AAD_RECORD: &[u8] = b"instance-A|record-1";
    const AAD_OTHER: &[u8] = b"instance-A|record-2";

    #[test]
    fn round_trip_recovers_plaintext() {
        let key = generate_data_key().unwrap();
        let sealed = seal(&key, b"national_id=12345678", AAD_RECORD).unwrap();
        let opened = open(&key, &sealed, AAD_RECORD).unwrap();
        assert_eq!(opened, b"national_id=12345678");
    }

    #[test]
    fn sealed_blob_carries_nonce_and_tag() {
        let key = generate_data_key().unwrap();
        let sealed = seal(&key, b"hello", AAD_RECORD).unwrap();
        assert_eq!(sealed.len(), NONCE_LEN + "hello".len() + TAG_LEN);
    }

    #[test]
    fn two_seals_of_the_same_plaintext_differ() {
        // A fresh nonce per call: identical plaintext must not produce
        // identical ciphertext, or the store would leak that two records hold
        // the same value.
        let key = generate_data_key().unwrap();
        let a = seal(&key, b"same", AAD_RECORD).unwrap();
        let b = seal(&key, b"same", AAD_RECORD).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn different_keys_do_not_decrypt_each_other() {
        let k1 = generate_data_key().unwrap();
        let k2 = generate_data_key().unwrap();
        let sealed = seal(&k1, b"secret", AAD_RECORD).unwrap();
        assert!(open(&k2, &sealed, AAD_RECORD).is_err());
    }

    #[test]
    fn modified_ciphertext_is_rejected() {
        let key = generate_data_key().unwrap();
        let mut sealed = seal(&key, b"national_id=12345678", AAD_RECORD).unwrap();
        let last = sealed.len() - 1;
        sealed[last] ^= 0x01; // flip one bit of the tag
        assert!(open(&key, &sealed, AAD_RECORD).is_err());
    }

    #[test]
    fn modified_plaintext_is_rejected() {
        let key = generate_data_key().unwrap();
        let mut sealed = seal(&key, b"national_id=12345678", AAD_RECORD).unwrap();
        sealed[NONCE_LEN] ^= 0x01;
        assert!(open(&key, &sealed, AAD_RECORD).is_err());
    }

    #[test]
    fn ciphertext_cannot_be_transplanted_to_another_record() {
        // The reason associated data is mandatory: without binding the record
        // identity, an attacker could move an encrypted datum between records.
        let key = generate_data_key().unwrap();
        let sealed = seal(&key, b"victim's national id", AAD_RECORD).unwrap();
        assert!(
            open(&key, &sealed, AAD_OTHER).is_err(),
            "must not decrypt under a different record's associated data"
        );
    }

    #[test]
    fn truncated_blob_is_rejected() {
        let key = generate_data_key().unwrap();
        let sealed = seal(&key, b"hi", AAD_RECORD).unwrap();
        assert!(open(&key, &sealed[..NONCE_LEN], AAD_RECORD).is_err());
        assert!(open(&key, &[], AAD_RECORD).is_err());
    }

    #[test]
    fn empty_plaintext_round_trips() {
        let key = generate_data_key().unwrap();
        let sealed = seal(&key, b"", AAD_RECORD).unwrap();
        assert!(open(&key, &sealed, AAD_RECORD).unwrap().is_empty());
    }

    #[test]
    fn generated_keys_differ() {
        assert_ne!(generate_data_key().unwrap(), generate_data_key().unwrap());
    }
}
