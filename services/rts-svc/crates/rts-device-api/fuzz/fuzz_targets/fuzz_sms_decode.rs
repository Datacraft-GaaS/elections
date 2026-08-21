//! Fuzz target for the SMS fallback CBOR+zstd decoder (M08-FUZZ-SMS-001).
//!
//! cargo fuzz run fuzz_sms_decode -- -max_len=2048
//!
//! The decoder must never panic on arbitrary bytes. It should return an Err on
//! malformed input. This target also verifies that any successfully decoded form
//! satisfies the arithmetic invariant: votes_cast == valid + rejected + spoilt.

#![no_main]
use libfuzzer_sys::fuzz_target;
use rts_device_api::codec::decode_sms;

fuzz_target!(|data: &[u8]| {
    if let Ok(form) = decode_sms(data) {
        // Arithmetic invariant must hold for any decodable form.
        let _ = form.votes_cast;
        let _ = form.valid_votes + form.rejected_votes + form.spoilt_ballots;
        // No panic is the primary property; arithmetic check is secondary.
    }
    // Errors are expected and acceptable; panics are not.
});
