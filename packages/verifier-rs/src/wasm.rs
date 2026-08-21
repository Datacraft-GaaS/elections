//! WASM exports via wasm-bindgen.
//!
//! All functions accept JSON strings for structured arguments and return
//! `Result<(), JsValue>` so JS callers get typed errors.
//!
//! Build with: `wasm-pack build --target web` from packages/verifier-rs/.

use wasm_bindgen::prelude::*;

use crate::{
    verify_bundle_hash, verify_consistency, verify_inclusion, verify_sth_signature,
    verify_witness_cosign, ConsistencyProof, InclusionProof, Sth, WitnessSig,
};

fn js_err(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Verify Merkle inclusion.
///
/// `json`: `{"sth":{...},"proof":{...},"leaf_hex":"<hex>"}`.
#[wasm_bindgen]
pub fn eel_verifier_inclusion_wasm(json: &str) -> Result<(), JsValue> {
    #[derive(serde::Deserialize)]
    struct Req {
        sth: Sth,
        proof: InclusionProof,
        leaf_hex: String,
    }
    let req: Req = serde_json::from_str(json).map_err(js_err)?;
    let leaf = hex::decode(&req.leaf_hex).map_err(js_err)?;
    verify_inclusion(&req.sth, &req.proof, &leaf).map_err(js_err)
}

/// Verify Merkle consistency.
///
/// `json`: `{"from_sth":{...},"to_sth":{...},"proof":{...}}`.
#[wasm_bindgen]
pub fn eel_verifier_consistency_wasm(json: &str) -> Result<(), JsValue> {
    #[derive(serde::Deserialize)]
    struct Req {
        from_sth: Sth,
        to_sth: Sth,
        proof: ConsistencyProof,
    }
    let req: Req = serde_json::from_str(json).map_err(js_err)?;
    verify_consistency(&req.from_sth, &req.to_sth, &req.proof).map_err(js_err)
}

/// Verify an STH Ed25519 signature.
///
/// `json`: `{"sth":{...}}`.
#[wasm_bindgen]
pub fn eel_verifier_sth_signature_wasm(json: &str) -> Result<(), JsValue> {
    #[derive(serde::Deserialize)]
    struct Req {
        sth: Sth,
    }
    let req: Req = serde_json::from_str(json).map_err(js_err)?;
    verify_sth_signature(&req.sth, None).map_err(js_err)
}

/// Verify witness cosignatures.
///
/// `json`: `{"sth":{...},"witness_sigs":[...]}`.
#[wasm_bindgen]
pub fn eel_verifier_witness_cosign_wasm(json: &str) -> Result<(), JsValue> {
    #[derive(serde::Deserialize)]
    struct Req {
        sth: Sth,
        witness_sigs: Vec<WitnessSig>,
    }
    let req: Req = serde_json::from_str(json).map_err(js_err)?;
    verify_witness_cosign(&req.sth, &req.witness_sigs).map_err(js_err)
}

/// Verify a bundle BLAKE3 hash.
///
/// `bundle_bytes`: raw bundle data.
/// `expected_root_hex`: 64-char hex string.
#[wasm_bindgen]
pub fn eel_verifier_bundle_hash_wasm(
    bundle_bytes: &[u8],
    expected_root_hex: &str,
) -> Result<(), JsValue> {
    verify_bundle_hash(bundle_bytes, expected_root_hex).map_err(js_err)
}
