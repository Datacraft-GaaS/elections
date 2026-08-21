//! BLAKE3 hashing for the eel platform.
//!
//! A thin wrapper around the `blake3` crate that returns fixed-size arrays.
//!
//! # Example
//!
//! ```
//! use eel_crypto_rs::blake3::{hash, keyed_hash};
//! let digest = hash(b"election data");
//! assert_eq!(digest.len(), 32);
//! ```

/// Compute a BLAKE3 hash of `input`.
///
/// Returns a 32-byte digest.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::blake3::hash;
/// let h = hash(b"data");
/// assert_eq!(h.len(), 32);
/// ```
pub fn hash(input: &[u8]) -> [u8; 32] {
    *blake3::hash(input).as_bytes()
}

/// Compute a keyed BLAKE3 hash (MAC) of `input` using the provided `key`.
///
/// `key` must be exactly 32 bytes.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::blake3::keyed_hash;
/// let key = [0u8; 32];
/// let h = keyed_hash(&key, b"data");
/// assert_eq!(h.len(), 32);
/// ```
pub fn keyed_hash(key: &[u8; 32], input: &[u8]) -> [u8; 32] {
    *blake3::keyed_hash(key, input).as_bytes()
}

/// Compute a BLAKE3 hash of `input` and return it as a lower-hex string.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::blake3::hash_hex;
/// let h = hash_hex(b"data");
/// assert_eq!(h.len(), 64);
/// ```
pub fn hash_hex(input: &[u8]) -> String {
    hex::encode(hash(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let h1 = hash(b"test");
        let h2 = hash(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_inputs_different_digests() {
        let h1 = hash(b"a");
        let h2 = hash(b"b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn known_empty_hash() {
        // BLAKE3("") is a well-known constant.
        let expected = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
        assert_eq!(hash_hex(b""), expected);
    }
}
