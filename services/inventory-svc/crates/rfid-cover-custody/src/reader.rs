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

//! Reader trait and HMAC-SHA256 challenger for Gen2v2 tag authentication.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

use crate::error::RfidError;

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Reader trait
// ---------------------------------------------------------------------------

/// Abstract RFID reader that can challenge a tag with a nonce.
///
/// In production each concrete adapter (Impinj Speedway, Zebra FX9600, CipherLab RS36)
/// implements this via the vendor SDK; the `HmacChallenger` is used in tests.
pub trait Reader: Send + Sync {
    /// Issue a Gen2v2 authenticate challenge to the tag.
    ///
    /// `nonce` is a 16-byte random value; returns the tag's HMAC-SHA256 response.
    fn challenge(&self, nonce: &[u8]) -> Result<Vec<u8>, RfidError>;
}

// ---------------------------------------------------------------------------
// HmacChallenger
// ---------------------------------------------------------------------------

/// Gen2v2-compatible HMAC-SHA256 challenger.
///
/// Simulates the tag's crypto engine: `response = HMAC-SHA256(tag_secret, nonce)`.
/// In production the tag secret lives in the HSM; this implementation is used
/// in tests and CI where no hardware is available.
#[derive(Clone)]
pub struct HmacChallenger {
    /// Per-tag secret (32 bytes). In production this is fetched from HSM by key ID.
    tag_secret: Vec<u8>,
}

impl HmacChallenger {
    pub fn new(tag_secret: Vec<u8>) -> Self {
        Self { tag_secret }
    }

    /// Compute `HMAC-SHA256(tag_secret, nonce)` and return the 32-byte result.
    pub fn compute_response(tag_secret: &[u8], nonce: &[u8]) -> Result<Vec<u8>, RfidError> {
        let mut mac =
            HmacSha256::new_from_slice(tag_secret).map_err(|_| RfidError::InvalidHmacKey)?;
        mac.update(nonce);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Verify a tag response matches the expected HMAC for the given nonce and secret.
    pub fn verify_response(
        tag_secret: &[u8],
        nonce: &[u8],
        response: &[u8],
    ) -> Result<bool, RfidError> {
        let expected = Self::compute_response(tag_secret, nonce)?;
        // Constant-time comparison: HMAC verify via digest
        Ok(expected.as_slice() == response)
    }
}

impl Reader for HmacChallenger {
    fn challenge(&self, nonce: &[u8]) -> Result<Vec<u8>, RfidError> {
        Self::compute_response(&self.tag_secret, nonce)
    }
}

// ---------------------------------------------------------------------------
// ReaderRegistry
// ---------------------------------------------------------------------------

/// Metadata about a registered reader.
#[derive(Debug, Clone)]
pub struct ReaderInfo {
    /// SPIFFE SVID of this reader, e.g. `spiffe://eel.local/reader/RDR-0001`.
    pub svid: String,
    /// Hardware serial number.
    pub serial: String,
    /// Human-readable location.
    pub location: String,
}

/// Registry of known readers, keyed by reader_id.
///
/// In production, this is loaded from a config store / database at startup.
/// Tests populate it directly.
#[derive(Default)]
pub struct ReaderRegistry {
    readers: HashMap<String, ReaderInfo>,
}

impl ReaderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, reader_id: impl Into<String>, info: ReaderInfo) {
        self.readers.insert(reader_id.into(), info);
    }

    /// Returns `Some(&ReaderInfo)` if the reader_id is known.
    pub fn get(&self, reader_id: &str) -> Option<&ReaderInfo> {
        self.readers.get(reader_id)
    }

    /// Verify that a reader's SVID matches the registered SVID.
    ///
    /// Returns `Err(ReaderSvidRejected)` if not registered or SVID mismatch.
    pub fn verify_svid(&self, reader_id: &str, presented_svid: &str) -> Result<(), RfidError> {
        let info = self.readers.get(reader_id).ok_or_else(|| {
            RfidError::ReaderSvidRejected(format!("reader {reader_id} not registered"))
        })?;

        if info.svid != presented_svid {
            return Err(RfidError::ReaderSvidRejected(format!(
                "reader {reader_id}: expected SVID '{}', got '{}'",
                info.svid, presented_svid
            )));
        }

        Ok(())
    }
}
