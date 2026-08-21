#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Attempt to parse arbitrary bytes as an Sth JSON and verify its signature.
    if let Ok(json) = std::str::from_utf8(data) {
        if let Ok(sth) = serde_json::from_str::<eel_verifier::Sth>(json) {
            // Verification result (Ok or Err) is fine; panics are not.
            let _ = eel_verifier::verify_sth_signature(&sth, None);
        }
    }
});
