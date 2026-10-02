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

//! Nonce challenge/response with 60-second freshness window.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};

use crate::error::AttestError;

/// Freshness window in seconds.
const NONCE_FRESHNESS_SECS: i64 = 60;

/// A 32-byte random nonce and its creation timestamp.
#[derive(Debug, Clone)]
pub struct Nonce {
    pub bytes: [u8; 32],
    pub created_at: DateTime<Utc>,
}

impl Nonce {
    /// Generate a fresh 32-byte random nonce using UUID v4 entropy.
    pub fn generate() -> Self {
        let a = uuid::Uuid::new_v4();
        let b = uuid::Uuid::new_v4();
        let mut bytes = [0u8; 32];
        bytes[..16].copy_from_slice(a.as_bytes());
        bytes[16..].copy_from_slice(b.as_bytes());
        Self {
            bytes,
            created_at: Utc::now(),
        }
    }

    /// Hex-encode the nonce bytes.
    pub fn hex(&self) -> String {
        hex::encode(self.bytes)
    }

    /// Return true if the nonce is still within the freshness window.
    pub fn is_fresh(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.created_at);
        age < Duration::seconds(NONCE_FRESHNESS_SECS)
    }
}

/// In-memory nonce store. Tracks issued nonces and prevents replay.
pub struct NonceStore {
    /// Maps hex nonce -> (Nonce, used flag).
    inner: Mutex<HashMap<String, (Nonce, bool)>>,
}

impl NonceStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Issue a new nonce.
    pub fn issue(&self) -> Nonce {
        let nonce = Nonce::generate();
        let mut map = self.inner.lock().expect("nonce store lock");
        map.insert(nonce.hex(), (nonce.clone(), false));
        // Evict stale entries to avoid unbounded growth.
        map.retain(|_, (n, _)| n.is_fresh());
        nonce
    }

    /// Consume a nonce: returns Ok if fresh and not yet used, else Err.
    pub fn consume(&self, hex_nonce: &str) -> Result<(), AttestError> {
        let mut map = self.inner.lock().expect("nonce store lock");
        match map.get_mut(hex_nonce) {
            None => Err(AttestError::NonceNotFound),
            Some((nonce, used)) => {
                if !nonce.is_fresh() {
                    return Err(AttestError::NonceExpired);
                }
                if *used {
                    return Err(AttestError::Replay);
                }
                *used = true;
                Ok(())
            }
        }
    }
}

impl Default for NonceStore {
    fn default() -> Self {
        Self::new()
    }
}
