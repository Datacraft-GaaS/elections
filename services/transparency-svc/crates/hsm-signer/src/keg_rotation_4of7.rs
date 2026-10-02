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

//! Freeze-window KEK rotation ceremony (HARD-17).
//!
//! During the election freeze window (EEL_FREEZE_WINDOW_FROM .. EEL_FREEZE_WINDOW_TO),
//! KEK rotation requires 4-of-7 custodian quorum instead of the standard 3-of-5.
//!
//! Reads two RFC-3339 timestamps from environment:
//!   EEL_FREEZE_WINDOW_FROM  (e.g. "2024-08-08T00:00:00Z")
//!   EEL_FREEZE_WINDOW_TO    (e.g. "2024-08-15T23:59:59Z")

use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::ceremony::{Ceremony, CeremonyStore, CeremonyUpdate};

/// Inclusive freeze window parsed from environment.
pub struct FreezeWindow {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

/// Parse EEL_FREEZE_WINDOW_FROM and EEL_FREEZE_WINDOW_TO from the environment.
///
/// Both variables must be present and valid RFC-3339 timestamps.
pub fn freeze_window_from_env() -> Result<FreezeWindow, String> {
    let from_raw = std::env::var("EEL_FREEZE_WINDOW_FROM")
        .map_err(|_| "EEL_FREEZE_WINDOW_FROM not set".to_string())?;
    let to_raw = std::env::var("EEL_FREEZE_WINDOW_TO")
        .map_err(|_| "EEL_FREEZE_WINDOW_TO not set".to_string())?;

    let from = DateTime::parse_from_rfc3339(&from_raw)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("EEL_FREEZE_WINDOW_FROM parse error: {e}"))?;
    let to = DateTime::parse_from_rfc3339(&to_raw)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("EEL_FREEZE_WINDOW_TO parse error: {e}"))?;

    if from > to {
        return Err(format!(
            "EEL_FREEZE_WINDOW_FROM ({from}) is after EEL_FREEZE_WINDOW_TO ({to})"
        ));
    }

    Ok(FreezeWindow { from, to })
}

/// Returns true when `now` falls within the inclusive freeze window.
pub fn is_in_freeze_window(w: &FreezeWindow, now: DateTime<Utc>) -> bool {
    now >= w.from && now <= w.to
}

/// Returns the (M, N) quorum parameters for the current context.
///
/// In-freeze: (4, 7). Normal: (3, 5).
pub fn quorum_params(in_freeze: bool) -> (u32, u32) {
    if in_freeze {
        (4, 7)
    } else {
        (3, 5)
    }
}

/// Ceremony coordinator that enforces freeze-window quorum escalation.
pub struct FreezeWindowKekRotation {
    pub store: Arc<CeremonyStore>,
    pub freeze_window: FreezeWindow,
}

impl FreezeWindowKekRotation {
    /// Construct from environment. Fails if env vars are missing or malformed.
    pub fn new(store: Arc<CeremonyStore>) -> Result<Self, String> {
        let freeze_window = freeze_window_from_env()?;
        Ok(Self {
            store,
            freeze_window,
        })
    }

    /// Open a KEK rotation ceremony with quorum determined by freeze-window status.
    ///
    /// Returns the newly created `Ceremony` and a broadcast receiver for updates.
    pub fn open_rotation_ceremony(
        &self,
        role: eel_crypto_rs::KeyRole,
        purpose: String,
    ) -> Result<(Ceremony, tokio::sync::broadcast::Receiver<CeremonyUpdate>), String> {
        let now = Utc::now();
        let in_freeze = is_in_freeze_window(&self.freeze_window, now);
        let (m, n) = quorum_params(in_freeze);
        Ok(self.store.open(role, purpose, m, n))
    }

    /// Return the effective (M, N) quorum for a given instant.
    pub fn effective_quorum(&self, now: DateTime<Utc>) -> (u32, u32) {
        quorum_params(is_in_freeze_window(&self.freeze_window, now))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ceremony::{CeremonyState, CeremonyStore};
    use chrono::Duration;
    use eel_crypto_rs::KeyRole;

    // Convenience: build a FreezeWindow directly without env vars.
    fn window(from: DateTime<Utc>, to: DateTime<Utc>) -> FreezeWindow {
        FreezeWindow { from, to }
    }

    // Build a store and FreezeWindowKekRotation with the given window, no env needed.
    fn make_fwkr(store: Arc<CeremonyStore>, fw: FreezeWindow) -> FreezeWindowKekRotation {
        FreezeWindowKekRotation {
            store,
            freeze_window: fw,
        }
    }

    // ---------------------------------------------------------------------------
    // Boundary tests (no store interaction needed)
    // ---------------------------------------------------------------------------

    #[test]
    fn test_freeze_boundary_exact_start() {
        let now = Utc::now();
        let fw = window(now, now + Duration::hours(1));
        assert!(is_in_freeze_window(&fw, now));
    }

    #[test]
    fn test_freeze_boundary_just_before() {
        let now = Utc::now();
        let start = now + Duration::seconds(1);
        let fw = window(start, start + Duration::hours(1));
        assert!(!is_in_freeze_window(&fw, now));
    }

    #[test]
    fn test_freeze_boundary_exact_end() {
        let now = Utc::now();
        let fw = window(now - Duration::hours(1), now);
        assert!(is_in_freeze_window(&fw, now));
    }

    #[test]
    fn test_freeze_boundary_just_after() {
        let now = Utc::now();
        let end = now - Duration::seconds(1);
        let fw = window(end - Duration::hours(1), end);
        assert!(!is_in_freeze_window(&fw, now));
    }

    // ---------------------------------------------------------------------------
    // Quorum selection via open_rotation_ceremony
    // ---------------------------------------------------------------------------

    #[test]
    fn test_4of7_in_freeze() {
        let now = Utc::now();
        let fw = window(now - Duration::hours(1), now + Duration::hours(1));
        let store = Arc::new(CeremonyStore::new());
        let fwkr = make_fwkr(Arc::clone(&store), fw);

        let (ceremony, _rx) = fwkr
            .open_rotation_ceremony(KeyRole::CycleKek, "freeze-rotation".into())
            .expect("open should succeed");

        assert_eq!(ceremony.quorum_m, 4, "expected M=4 in freeze window");
        assert_eq!(ceremony.quorum_n, 7, "expected N=7 in freeze window");
    }

    #[test]
    fn test_3of5_outside_freeze() {
        let past_from = DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let past_to = DateTime::parse_from_rfc3339("2020-01-07T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);
        let fw = window(past_from, past_to);
        let store = Arc::new(CeremonyStore::new());
        let fwkr = make_fwkr(Arc::clone(&store), fw);

        let (ceremony, _rx) = fwkr
            .open_rotation_ceremony(KeyRole::CycleKek, "normal-rotation".into())
            .expect("open should succeed");

        assert_eq!(ceremony.quorum_m, 3, "expected M=3 outside freeze window");
        assert_eq!(ceremony.quorum_n, 5, "expected N=5 outside freeze window");
    }

    // ---------------------------------------------------------------------------
    // Quorum threshold enforcement: 3 approvals are insufficient for a 4-of-7 ceremony
    // ---------------------------------------------------------------------------

    #[test]
    fn test_3of7_rejects() {
        use ed25519_dalek::{Signer, SigningKey};
        use rand_core::OsRng;

        let store = Arc::new(CeremonyStore::new());

        // Open a 4-of-7 ceremony directly.
        let (ceremony, _rx) = store.open(KeyRole::CycleKek, "freeze-rotation".into(), 4, 7);
        let ceremony_id = ceremony.id.clone();
        let op_hash = ceremony.operation_hash.clone();

        // Register 3 custodians and have each approve.
        for _ in 0..3 {
            let signing_key = SigningKey::generate(&mut OsRng);
            let verifying_key = signing_key.verifying_key();
            let custodian =
                store.register_custodian("test-custodian".to_string(), verifying_key.to_bytes());

            let mut message = ceremony_id.as_bytes().to_vec();
            message.extend_from_slice(op_hash.as_bytes());
            let sig = signing_key.sign(&message);

            store
                .approve(&ceremony_id, &custodian.id, sig.to_bytes().as_ref())
                .expect("approve should succeed");
        }

        // Fetch the updated ceremony and assert it is still awaiting quorum.
        let updated = store.get(&ceremony_id).expect("ceremony must exist");
        match &updated.state {
            CeremonyState::AwaitingQuorum { approvals } => {
                assert_eq!(approvals.len(), 3, "expected 3 approvals recorded");
                assert!(
                    (approvals.len() as u32) < updated.quorum_m,
                    "3 approvals must be below quorum_m=4"
                );
            }
            other => panic!(
                "expected AwaitingQuorum after 3/4 approvals, got {:?}",
                other
            ),
        }
    }
}
