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

use crate::IcaoError;
use sha2::{Digest, Sha256};

/// Selects which key to derive: encryption or MAC computation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyMode {
    Encryption,
    MacComputation,
}

impl KeyMode {
    fn mode_byte(self) -> u8 {
        match self {
            KeyMode::Encryption => 0x01,
            KeyMode::MacComputation => 0x02,
        }
    }
}

/// Derive a 16-byte BAC session key from MRZ info.
///
/// Production ICAO 9303 Part 11 specifies SHA-1; this implementation uses
/// SHA-256 (truncated to 16 bytes) because sha1 is not in the workspace
/// dependency set. Wire in the sha1 crate and swap the hasher for strict
/// interoperability with ICAO-conformant chips.
///
/// Algorithm: SHA-256(mrz_info || 0x00 0x00 0x00 mode_byte)[0..16]
/// then adjust parity bits per TDES key schedule.
pub fn derive_key(mrz_info: &[u8], mode: KeyMode) -> Result<[u8; 16], IcaoError> {
    let mut hasher = Sha256::new();
    hasher.update(mrz_info);
    hasher.update([0x00, 0x00, 0x00, mode.mode_byte()]);
    let hash = hasher.finalize();

    let mut key = [0u8; 16];
    key.copy_from_slice(&hash[..16]);

    // Adjust parity bits: set bit 0 of each byte to odd parity.
    for byte in key.iter_mut() {
        let ones = byte.count_ones();
        if ones % 2 == 0 {
            *byte ^= 0x01;
        }
    }

    Ok(key)
}

/// PACE key derivation stub.
///
/// Wire in the actual PACE implementation (ICAO 9303 Part 11 section 4.2)
/// for production use. This stub returns a zeroed 16-byte placeholder.
pub fn pace_derive_key(
    _document_number: &str,
    _birth_date: &str,
    _expiry_date: &str,
) -> Result<Vec<u8>, IcaoError> {
    Ok(vec![0u8; 16])
}
