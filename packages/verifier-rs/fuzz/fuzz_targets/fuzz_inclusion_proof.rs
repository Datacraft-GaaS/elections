#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Parse as JSON: {"sth":{...},"proof":{...},"leaf_hex":"..."}
    // Any parse or verification error is acceptable; panics are not.
    if let Ok(json) = std::str::from_utf8(data) {
        #[derive(serde::Deserialize)]
        struct Req {
            sth: eel_verifier::Sth,
            proof: eel_verifier::InclusionProof,
            leaf_hex: String,
        }
        if let Ok(req) = serde_json::from_str::<Req>(json) {
            if let Ok(leaf) = hex::decode(&req.leaf_hex) {
                let _ = eel_verifier::verify_inclusion(&req.sth, &req.proof, &leaf);
            }
        }
    }
});
