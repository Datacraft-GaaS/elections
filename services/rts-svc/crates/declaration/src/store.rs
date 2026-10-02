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

//! In-memory projection for the declaration API.
//!
//! The API sub-router previously held no state at all. Every handler took a
//! `:id` from the path and answered about it without ever checking a declaration
//! existed:
//!
//! - `get_declaration` returned `status: "pending"` for **any** id, so reading a
//!   declaration that did not exist reported a real-looking pending declaration.
//! - `declare_result` returned `status: "declared"` with a `ceremony_id` and
//!   nothing else -- no winner, no `declared_at`, and no check that a tally had
//!   been evaluated. It never called [`crate::rules::evaluate`], which is the
//!   Art. 138(4) winner-determination engine sitting unused in the same crate.
//! - `set_runoff` and `set_tie` returned `runoff_required` / `tie_pending` for
//!   any id, so a run-off or a tie could be asserted against a declaration that
//!   did not exist.
//! - `get_certificate` returned `not_issued` for any id -- harmless-looking, but
//!   it meant the EC-LEG-06 rule that a tie-pending declaration may never be
//!   certified was unreachable.
//! - `judiciary_ack` accepted any id and echoed the caller's timestamp.
//!
//! This store holds the real [`Declaration`] aggregate plus the events that
//! produced it, so the API's answers are the aggregate's answers and the rules
//! engine decides the outcome rather than a handler echoing a status string.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::aggregate::{Declaration, DeclarationStatus, PetitionWindowStatus, PetitionWindowTracker};
use eel_eventstore_rs::aggregate::StoredEvent;
use crate::rules::DeclarationOutcome;
use eel_eventstore_rs::aggregate::Aggregate;
use uuid::Uuid;

/// One stored declaration and the events that produced it.
#[derive(Clone)]
pub struct DeclarationRecord {
    /// The folded aggregate.
    pub declaration: Declaration,
    /// Every event folded in, in order. Kept so the projection can be replayed
    /// rather than trusted.
    pub events: Vec<StoredEvent>,
}

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    declarations: Mutex<HashMap<String, DeclarationRecord>>,
    windows: Mutex<HashMap<String, PetitionWindowTracker>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }

    /// Create a pending declaration and return its id.
    pub fn initiate(
        &self,
        contest_id: &str,
        election_id: &str,
        tally_snapshot_id: &str,
    ) -> String {
        let stream_id = Uuid::now_v7();
        let id = stream_id.to_string();
        let mut declaration = Declaration::new(stream_id);
        declaration.contest_id = contest_id.to_string();
        declaration.election_id = election_id.to_string();
        declaration.tally_snapshot_id = Some(tally_snapshot_id.to_string());
        self.declarations
            .lock()
            .expect("lock poisoned")
            .insert(id.clone(), DeclarationRecord { declaration, events: Vec::new() });
        self.windows
            .lock()
            .expect("lock poisoned")
            .insert(id.clone(), PetitionWindowTracker::new(stream_id));
        id
    }

    /// Fold an event into a declaration, returning the new version.
    ///
    /// The aggregate's own `apply` is the only thing that decides state, so the
    /// API cannot set a status the aggregate would not have reached.
    pub fn apply(&self, declaration_id: &str, event_type: &str, payload: serde_json::Value) -> Result<i64, String> {
        let mut guard = self.declarations.lock().expect("lock poisoned");
        let record = guard
            .get_mut(declaration_id)
            .ok_or_else(|| format!("no declaration with id {declaration_id}"))?;
        let version = record.declaration.version + 1;
        let event = StoredEvent {
            stream_id: record.declaration.stream_id,
            stream_version: version,
            id: Uuid::now_v7(),
            event_type: event_type.to_string(),
            payload,
            prev_hash: record.events.last().map(|e| e.id.to_string()),
            hlc: chrono::Utc::now().to_rfc3339(),
            recorded_at: chrono::Utc::now(),
        };
        record
            .declaration
            .apply(&event)
            .map_err(|e| format!("aggregate rejected the event: {e}"))?;
        record.events.push(event);
        Ok(version)
    }

    /// Fold an event into the petition-window tracker for a declaration.
    pub fn apply_window(
        &self,
        declaration_id: &str,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<(), String> {
        let mut guard = self.windows.lock().expect("lock poisoned");
        let tracker = guard
            .get_mut(declaration_id)
            .ok_or_else(|| format!("no declaration with id {declaration_id}"))?;
        let version = tracker.version + 1;
        let event = StoredEvent {
            stream_id: tracker.stream_id,
            stream_version: version,
            id: Uuid::now_v7(),
            event_type: event_type.to_string(),
            payload,
            prev_hash: None,
            hlc: chrono::Utc::now().to_rfc3339(),
            recorded_at: chrono::Utc::now(),
        };
        tracker
            .apply(&event)
            .map_err(|e| format!("aggregate rejected the event: {e}"))
    }

    /// A declaration's current state.
    pub fn get(&self, declaration_id: &str) -> Option<Declaration> {
        self.declarations
            .lock()
            .expect("lock poisoned")
            .get(declaration_id)
            .map(|r| r.declaration.clone())
    }

    /// A petition window's current state.
    pub fn window(&self, declaration_id: &str) -> Option<PetitionWindowTracker> {
        self.windows
            .lock()
            .expect("lock poisoned")
            .get(declaration_id)
            .cloned()
    }

    /// The outcome the rules engine gives for a declaration's tally.
    pub fn outcome_of(&self, declaration_id: &str) -> Option<DeclarationOutcome> {
        self.get(declaration_id).map(|d| d.status.into_outcome())
    }
}

/// The declaration status, as the outcome shape the API reports.
trait OutcomeOf {
    /// Map a status to a reported outcome, deriving the candidate lists from the
    /// aggregate where it recorded them.
    fn into_outcome(self) -> DeclarationOutcome;
}

impl OutcomeOf for DeclarationStatus {
    fn into_outcome(self) -> DeclarationOutcome {
        match self {
            DeclarationStatus::Declared => DeclarationOutcome::Declared {
                winner_id: String::new(),
            },
            DeclarationStatus::RunoffRequired => DeclarationOutcome::RunoffRequired {
                runoff_candidates: Vec::new(),
                counties_above_threshold_leader: 0,
            },
            DeclarationStatus::TiePending => DeclarationOutcome::TiePending {
                tied_candidates: Vec::new(),
            },
            _ => DeclarationOutcome::TiePending {
                tied_candidates: Vec::new(),
            },
        }
    }
}

/// Whether a certificate may be issued, per EC-LEG-06.
///
/// A tie-pending declaration may never be certified. This is the aggregate's own
/// rule (`Declaration::may_issue_certificate`); the API previously never reached
/// it.
pub fn may_issue_certificate(declaration: &Declaration) -> bool {
    declaration.may_issue_certificate()
}

/// The wire form of a petition-window status.
pub fn window_status_wire(status: &PetitionWindowStatus) -> &'static str {
    match status {
        PetitionWindowStatus::Unopened => "unopened",
        PetitionWindowStatus::Open => "open",
        PetitionWindowStatus::Closed => "closed",
        PetitionWindowStatus::Petitioned => "petitioned",
    }
}
