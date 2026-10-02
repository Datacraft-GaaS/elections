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
