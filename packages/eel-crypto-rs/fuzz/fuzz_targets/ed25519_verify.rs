//! Fuzz target: Ed25519 verify with malformed inputs.
//!
//! Feeds random bytes as key material, signature bytes, and message content.
//! Must never panic; errors are expected and correct.

#![no_main]
use eel_crypto_rs::ed25519::{signature_from_bytes, verify, verifying_key_from_bytes};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 96 {
        return;
    }

    let key_bytes: &[u8; 32] = data[..32].try_into().unwrap();
    let sig_bytes: &[u8; 64] = data[32..96].try_into().unwrap();
    let message = &data[96..];

    // Constructing a verifying key or signature from random bytes may fail;
    // that is expected.  The invariant is "no panic".
    if let Ok(vk) = verifying_key_from_bytes(key_bytes) {
        if let Ok(sig) = signature_from_bytes(sig_bytes) {
            // Verification may succeed or fail; either is fine.
            let _ = verify(&vk, message, &sig);
        }
    }
});
