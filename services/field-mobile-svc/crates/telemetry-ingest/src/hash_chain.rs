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

//! BLAKE3 hash chain over eel_telemetry_signals.
//!
//! Each signal hashes `prev_hash || canonical_json(signal_data)`.
//! A silent rewrite breaks the chain; `verify_chain` reconstructs and rejects.

use crate::domain::TelemetrySignal;
use crate::error::TelemetryError;

/// Compute `BLAKE3(prev_hash || canonical_json(signal))`.
///
/// `prev_hash` is `&[]` for the genesis (first) row of a device.
pub fn compute_this_hash(prev_hash: &[u8], signal: &TelemetrySignal) -> [u8; 32] {
    let canonical = serde_json::json!({
        "signal_id":          signal.signal_id.to_string(),
        "device_id":          signal.device_id.to_string(),
        "session_id":         signal.session_id.map(|id| id.to_string()),
        "class":              signal.class.as_str(),
        "occurred_at":        signal.occurred_at.to_rfc3339(),
        "monotonic_counter":  signal.monotonic_counter,
        "payload":            signal.payload.clone(),
    });

    let canonical_bytes = serde_json::to_vec(&canonical).unwrap_or_else(|_| b"{}".to_vec());

    let mut hasher = blake3::Hasher::new();
    if !prev_hash.is_empty() {
        hasher.update(prev_hash);
    }
    hasher.update(&canonical_bytes);
    *hasher.finalize().as_bytes()
}

/// Verify the hash chain for an ordered slice of signals (oldest first).
///
/// Returns `Ok(())` if every `this_hash` matches the recomputed value.
/// Returns `Err(ChainBroken)` at the first mismatch.
pub fn verify_chain(signals: &[TelemetrySignal]) -> Result<(), TelemetryError> {
    if signals.is_empty() {
        return Ok(());
    }

    let mut prev: &[u8] = &[];
    for s in signals {
        let expected = compute_this_hash(prev, s);
        if s.this_hash.as_slice() != expected.as_slice() {
            return Err(TelemetryError::ChainBroken {
                signal_id: s.signal_id.to_string(),
                expected: hex::encode(expected),
                computed: hex::encode(&s.this_hash),
            });
        }
        prev = &s.this_hash;
    }

    Ok(())
}
