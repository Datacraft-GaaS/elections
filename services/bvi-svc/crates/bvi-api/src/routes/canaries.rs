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

//! Canary voter records (N3): register-enumeration tripwires.
//!
//! Synthetic voter ids are configured via `EEL_VOTER_CANARIES` (comma-
//! separated). Any station lookup that resolves one of these ids — found or
//! not-found — proves someone is walking the register. Hits are logged (with
//! the canary identity redacted, never real voter data) and buffered in an
//! in-process ring so operators can drain them from
//! GET /api/v1/bvi/anomalies/canary-hits. The lookup response itself is
//! completely unchanged: callers cannot distinguish a canary probe.

use std::sync::{Mutex, OnceLock};

use axum::{extract::State, http::HeaderMap, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{error::ApiError, idem, state::AppState, store::BviStore};

const INSTANCE: &str = "/api/v1/bvi/anomalies/canary-hits";

/// Maximum retained hits in the in-process ring buffer.
const RING_CAP: usize = 1000;

/// One buffered canary trip. Deliberately carries no voter identifiers.
#[derive(Debug, Clone, Serialize)]
pub struct CanaryHit {
    pub at: DateTime<Utc>,
    /// Whether the underlying lookup was "found" or "not_found".
    pub lookup_outcome: String,
}

static REGISTRY: OnceLock<Vec<String>> = OnceLock::new();
static RING: OnceLock<Mutex<Vec<CanaryHit>>> = OnceLock::new();

/// Process-wide canary registry parsed once from `EEL_VOTER_CANARIES`.
pub fn registry() -> &'static [String] {
    REGISTRY.get_or_init(|| {
        std::env::var("EEL_VOTER_CANARIES")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    })
}

fn ring() -> &'static Mutex<Vec<CanaryHit>> {
    RING.get_or_init(|| Mutex::new(Vec::new()))
}

/// Case-sensitive exact membership test against the configured canary ids.
pub fn canary_hit(registry: &[String], voter_id: &str) -> bool {
    registry.iter().any(|c| c == voter_id)
}

/// Redacted, stable log identifier for a canary (sha256 prefix): the raw
/// synthetic id never appears in logs, and real voter data never will either.
fn redact(canary_id: &str) -> String {
    let digest = Sha256::digest(canary_id.as_bytes());
    format!("<canary:{}>", hex::encode(&digest[..6]))
}

/// Push a hit onto the ring, dropping the oldest entries past `cap`.
pub fn record_canary(ring: &Mutex<Vec<CanaryHit>>, cap: usize, outcome: &str) {
    let mut buf = ring.lock().expect("canary ring poisoned");
    buf.push(CanaryHit {
        at: Utc::now(),
        lookup_outcome: outcome.to_string(),
    });
    if buf.len() > cap {
        let overflow = buf.len() - cap;
        buf.drain(..overflow);
    }
}

/// Shared hook for the voter-station handler. Runs on BOTH lookup outcomes;
/// never alters the response seen by the caller.
pub fn observe_lookup(voter_id: &str, lookup_outcome: &str) {
    if !canary_hit(registry(), voter_id) {
        return;
    }
    tracing::error!(
        voter_id = %redact(voter_id),
        lookup_outcome,
        "CANARY HIT: register enumeration detected"
    );
    record_canary(ring(), RING_CAP, lookup_outcome);
}

#[derive(Debug, Serialize)]
pub struct CanaryHitsResponse {
    pub hits: Vec<CanaryHit>,
    pub total: usize,
}

/// GET /api/v1/bvi/anomalies/canary-hits: atomically drain all buffered hits.
pub async fn get_canary_hits<S: BviStore>(
    State(_state): State<AppState<S>>,
    headers: HeaderMap,
) -> Result<Json<CanaryHitsResponse>, ApiError> {
    idem::require_bearer(&headers, INSTANCE)?;
    let hits = std::mem::take(&mut *ring().lock().expect("canary ring poisoned"));
    let total = hits.len();
    Ok(Json(CanaryHitsResponse { hits, total }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_requires_exact_case_sensitive_match() {
        let reg = vec!["CANARY-001".to_string()];
        assert!(canary_hit(&reg, "CANARY-001"));
        assert!(!canary_hit(&reg, "canary-001"));
        assert!(!canary_hit(&reg, "CANARY-002"));
        assert!(!canary_hit(&reg, ""));
    }

    #[test]
    fn empty_registry_never_matches() {
        assert!(!canary_hit(&[], "anything"));
    }

    #[test]
    fn ring_caps_at_capacity_dropping_oldest() {
        let ring: Mutex<Vec<CanaryHit>> = Mutex::new(Vec::new());
        for i in 0..7 {
            record_canary(&ring, 5, &format!("hit{i}"));
        }
        let buf = ring.lock().unwrap();
        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0].lookup_outcome, "hit2");
        assert_eq!(buf[4].lookup_outcome, "hit6");
    }

    #[test]
    fn drain_empties_ring() {
        let ring: Mutex<Vec<CanaryHit>> = Mutex::new(Vec::new());
        record_canary(&ring, 10, "found");
        record_canary(&ring, 10, "not_found");
        let drained = std::mem::take(&mut *ring.lock().unwrap());
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].lookup_outcome, "found");
        assert!(ring.lock().unwrap().is_empty());
    }
}
