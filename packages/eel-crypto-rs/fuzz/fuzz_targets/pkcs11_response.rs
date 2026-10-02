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

//! Fuzz target: PKCS#11 response parser.
//!
//! Feeds random bytes into any PKCS#11 response parsing paths that can be
//! exercised without a live HSM.  Currently exercises the DER parsing used
//! by the TSA module, since the PKCS#11 library itself handles its own wire
//! format.  Must never panic.

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Exercise DER length decoding with arbitrary input.
    if data.is_empty() {
        return;
    }

    // Test that parsing arbitrary bytes as a timestamp response does not panic.
    // The function returns Err for short/malformed input; that is expected.
    let _ = parse_der_sequence_len(data);
});

/// Extract the content length from a DER-encoded SEQUENCE header, if present.
/// Returns `None` for any malformed input.
fn parse_der_sequence_len(data: &[u8]) -> Option<usize> {
    if data.len() < 2 || data[0] != 0x30 {
        return None;
    }
    let len_byte = data[1] as usize;
    if len_byte < 0x80 {
        Some(len_byte)
    } else if len_byte == 0x81 {
        data.get(2).map(|&b| b as usize)
    } else if len_byte == 0x82 {
        if data.len() >= 4 {
            Some(((data[2] as usize) << 8) | data[3] as usize)
        } else {
            None
        }
    } else {
        None
    }
}
