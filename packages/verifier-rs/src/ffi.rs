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

//! C ABI exports for Dart FFI.
//!
//! Each function accepts JSON-encoded arguments and returns 0 on success or
//! -1 on error.  Call `eel_verifier_last_error_message()` to retrieve the
//! error string (valid until the next call on the same thread).
//!
//! All C symbols carry the `eel_verifier_` prefix (ADR-0005).

// FFI boundary by definition: raw pointers, no_mangle exports, and unsafe
// blocks are the entire purpose of this module. The workspace denies
// `unsafe_code` via RUSTFLAGS; this scoped opt-out (in-source `allow`
// overrides command-line `deny`) keeps the rest of the tree under the ban.
// Every export documents its `# Safety` contract; see below.
#![allow(unsafe_code)]

use std::cell::RefCell;
use std::ffi::{c_char, CString};
use std::slice;

use crate::{
    verify_bundle_hash, verify_consistency, verify_inclusion, verify_sth_signature,
    verify_witness_cosign, ConsistencyProof, InclusionProof, Sth, WitnessSig,
};

// ---------------------------------------------------------------------------
// Thread-local error buffer
// ---------------------------------------------------------------------------

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(msg).ok();
    });
}

/// Return a pointer to the last error message string on this thread.
///
/// The pointer is valid until the next `eel_verifier_*` call on this thread.
/// Returns NULL if no error has occurred.
#[no_mangle]
pub extern "C" fn eel_verifier_last_error_message() -> *const c_char {
    LAST_ERROR.with(|e| match &*e.borrow() {
        Some(s) => s.as_ptr(),
        None => std::ptr::null(),
    })
}

// ---------------------------------------------------------------------------
// JSON request types (internal to this module)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct InclusionRequest {
    sth: Sth,
    proof: InclusionProof,
    leaf_hex: String,
}

#[derive(serde::Deserialize)]
struct ConsistencyRequest {
    from_sth: Sth,
    to_sth: Sth,
    proof: ConsistencyProof,
}

#[derive(serde::Deserialize)]
struct SthRequest {
    sth: Sth,
}

#[derive(serde::Deserialize)]
struct WitnessRequest {
    sth: Sth,
    witness_sigs: Vec<WitnessSig>,
}

// ---------------------------------------------------------------------------
// FFI helpers
// ---------------------------------------------------------------------------

unsafe fn json_bytes<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts(ptr, len) })
    }
}

macro_rules! ffi_run {
    ($expr:expr) => {
        match $expr {
            Ok(()) => 0i32,
            Err(e) => {
                set_last_error(&e.to_string());
                -1i32
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Exported functions
// ---------------------------------------------------------------------------

/// Verify Merkle inclusion.
///
/// `json` must be `{"sth":{...},"proof":{...},"leaf_hex":"<hex>"}`.
///
/// # Safety
///
/// `json_ptr` must point to `json_len` readable bytes for the
/// duration of the call; the buffer must remain valid and
/// unmodified while this function runs.
#[no_mangle]
pub unsafe extern "C" fn eel_verifier_inclusion(json_ptr: *const u8, json_len: usize) -> i32 {
    let bytes = match unsafe { json_bytes(json_ptr, json_len) } {
        Some(b) => b,
        None => {
            set_last_error("null json pointer");
            return -1;
        }
    };
    ffi_run!((|| -> Result<(), crate::VerifierError> {
        let req: InclusionRequest = serde_json::from_slice(bytes)?;
        let leaf = hex::decode(&req.leaf_hex)?;
        verify_inclusion(&req.sth, &req.proof, &leaf)
    })())
}

/// Verify Merkle consistency.
///
/// `json` must be `{"from_sth":{...},"to_sth":{...},"proof":{...}}`.
///
/// # Safety
///
/// `json_ptr` must point to `json_len` readable bytes for the
/// duration of the call; the buffer must remain valid and
/// unmodified while this function runs.
#[no_mangle]
pub unsafe extern "C" fn eel_verifier_consistency(json_ptr: *const u8, json_len: usize) -> i32 {
    let bytes = match unsafe { json_bytes(json_ptr, json_len) } {
        Some(b) => b,
        None => {
            set_last_error("null json pointer");
            return -1;
        }
    };
    ffi_run!((|| -> Result<(), crate::VerifierError> {
        let req: ConsistencyRequest = serde_json::from_slice(bytes)?;
        verify_consistency(&req.from_sth, &req.to_sth, &req.proof)
    })())
}

/// Verify an STH Ed25519 signature.
///
/// `json` must be `{"sth":{...}}`.
///
/// # Safety
///
/// `json_ptr` must point to `json_len` readable bytes for the
/// duration of the call; the buffer must remain valid and
/// unmodified while this function runs.
#[no_mangle]
pub unsafe extern "C" fn eel_verifier_sth_signature(json_ptr: *const u8, json_len: usize) -> i32 {
    let bytes = match unsafe { json_bytes(json_ptr, json_len) } {
        Some(b) => b,
        None => {
            set_last_error("null json pointer");
            return -1;
        }
    };
    ffi_run!((|| -> Result<(), crate::VerifierError> {
        let req: SthRequest = serde_json::from_slice(bytes)?;
        verify_sth_signature(&req.sth, None)
    })())
}

/// Verify witness cosignatures.
///
/// `json` must be `{"sth":{...},"witness_sigs":[...]}`.
///
/// # Safety
///
/// `json_ptr` must point to `json_len` readable bytes for the
/// duration of the call; the buffer must remain valid and
/// unmodified while this function runs.
#[no_mangle]
pub unsafe extern "C" fn eel_verifier_witness_cosign(json_ptr: *const u8, json_len: usize) -> i32 {
    let bytes = match unsafe { json_bytes(json_ptr, json_len) } {
        Some(b) => b,
        None => {
            set_last_error("null json pointer");
            return -1;
        }
    };
    ffi_run!((|| -> Result<(), crate::VerifierError> {
        let req: WitnessRequest = serde_json::from_slice(bytes)?;
        verify_witness_cosign(&req.sth, &req.witness_sigs)
    })())
}

/// Verify a bundle BLAKE3 hash.
///
/// `data_ptr`/`data_len`: raw bundle bytes.
/// `expected_root_ptr`/`expected_root_len`: UTF-8 hex string (64 chars).
///
/// # Safety
/// `data_ptr` must point to `data_len` readable bytes, and `expected_root_ptr`
/// to `expected_root_len` readable bytes. Both pointers must be non-null when
/// their length is non-zero. The caller must keep the buffers alive for the
/// duration of the call; this function does not retain them.
#[no_mangle]
pub unsafe extern "C" fn eel_verifier_bundle_hash(
    data_ptr: *const u8,
    data_len: usize,
    expected_root_ptr: *const u8,
    expected_root_len: usize,
) -> i32 {
    let data = match unsafe { json_bytes(data_ptr, data_len) } {
        Some(b) => b,
        None => {
            set_last_error("null data pointer");
            return -1;
        }
    };
    let expected_bytes = match unsafe { json_bytes(expected_root_ptr, expected_root_len) } {
        Some(b) => b,
        None => {
            set_last_error("null expected_root pointer");
            return -1;
        }
    };
    let expected_hex = match std::str::from_utf8(expected_bytes) {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("expected_root not valid UTF-8: {e}"));
            return -1;
        }
    };
    ffi_run!(verify_bundle_hash(data, expected_hex))
}
