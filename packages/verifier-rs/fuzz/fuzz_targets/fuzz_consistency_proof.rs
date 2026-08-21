#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Parse as JSON: {"from_sth":{...},"to_sth":{...},"proof":{...}}
    if let Ok(json) = std::str::from_utf8(data) {
        #[derive(serde::Deserialize)]
        struct Req {
            from_sth: eel_verifier::Sth,
            to_sth: eel_verifier::Sth,
            proof: eel_verifier::ConsistencyProof,
        }
        if let Ok(req) = serde_json::from_str::<Req>(json) {
            let _ = eel_verifier::verify_consistency(&req.from_sth, &req.to_sth, &req.proof);
        }
    }
});
