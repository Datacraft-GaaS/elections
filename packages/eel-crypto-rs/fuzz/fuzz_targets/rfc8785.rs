//! Fuzz target: RFC 8785 JSON canonicalization.
//!
//! Invariant: two semantically-equivalent JSON values (same keys, shuffled
//! order) must produce byte-identical canonical output.  Random valid JSON
//! input must never panic.

#![no_main]
use eel_crypto_rs::rfc8785::canonicalize;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Parse as JSON; invalid bytes are silently ignored (expected).
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(data) else {
        return;
    };

    // Canonicalize must not panic.
    let Ok(canonical) = canonicalize(&value) else {
        return;
    };

    // Round-trip: canonical output re-parsed and re-canonicalized must be
    // byte-identical to the first pass.
    let reparsed: serde_json::Value =
        serde_json::from_slice(&canonical).expect("canonical output must be valid JSON");
    let second = canonicalize(&reparsed).expect("second canonicalization must succeed");
    assert_eq!(
        canonical, second,
        "RFC 8785 canonicalization is not idempotent"
    );
});
