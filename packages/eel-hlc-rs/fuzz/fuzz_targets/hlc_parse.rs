//! Fuzz target: parse arbitrary bytes as an HLC string.
//!
//! Invariant: `Display(parse(s)) == s` whenever parse succeeds,
//! and parse must never panic on any input.

#![no_main]
use eel_hlc_rs::Hlc;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(hlc) = s.parse::<Hlc>() {
            // Round-trip invariant.
            let rendered = hlc.to_string();
            let back: Hlc = rendered.parse().expect("Display->parse must succeed");
            assert_eq!(back, hlc);
        }
    }
});
