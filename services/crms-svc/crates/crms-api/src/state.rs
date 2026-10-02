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

//! In-memory application state for the CRMS API.
//!
//! A real deployment replaces these `Mutex<HashMap>` registries with the
//! CRMS persistence layer; the handler surface is unchanged. Guarded maps
//! keep each aggregate independent so a contended aggregate never blocks the
//! rest of the service.

use std::collections::HashMap;
use std::sync::Mutex;

use crms_domain::ballot::BallotProof;
use crms_domain::candidate::Candidate;
use crms_domain::clearance::ClearanceDecision;
use crms_domain::coalition::CoalitionRegistration;
use crms_domain::events::DomainEvent;
use crms_domain::gazette::Gazette;
use crms_domain::nomination::Nomination;
use crms_domain::party::PoliticalParty;
use crms_domain::payment::FeePayment;
use crms_domain::payment::FeeSchedule;
use crms_domain::primaries::PrimaryDispute;
use crms_domain::primaries::PrimaryResult;
use crms_domain::symbol::PartySymbol;

use crate::spec::BallotProof as SpecBallotProof;
use uuid::Uuid;

/// Process-local registries backing the CRMS handlers.
#[derive(Default)]
pub struct AppState {
    /// Candidates by id.
    pub candidates: Mutex<HashMap<Uuid, Candidate>>,
    /// Nominations by id.
    pub nominations: Mutex<HashMap<Uuid, Nomination>>,
    /// Clearance decisions in insertion order.
    pub clearances: Mutex<Vec<ClearanceDecision>>,
    /// Gazettes by id.
    pub gazettes: Mutex<HashMap<Uuid, Gazette>>,
    /// Ballot proofs by id.
    pub ballots: Mutex<HashMap<Uuid, BallotProof>>,
    /// Parties by id.
    pub parties: Mutex<HashMap<Uuid, PoliticalParty>>,
    /// Coalitions by id.
    pub coalitions: Mutex<HashMap<Uuid, CoalitionRegistration>>,
    /// Primary results by id.
    pub primary_results: Mutex<HashMap<Uuid, PrimaryResult>>,
    /// Primary disputes in insertion order.
    pub disputes: Mutex<Vec<PrimaryDispute>>,
    /// Fee schedules in insertion order.
    pub fee_schedules: Mutex<Vec<FeeSchedule>>,
    /// Fee payments by id.
    pub payments: Mutex<HashMap<Uuid, FeePayment>>,
    /// Party symbols by id.
    pub symbols: Mutex<HashMap<Uuid, PartySymbol>>,
    /// Domain events in insertion order.
    pub events: Mutex<Vec<DomainEvent>>,
    /// Spec-facing ballot proofs (governance/nomination ballot-proofs surface).
    pub spec_ballot_proofs: Mutex<HashMap<String, SpecBallotProof>>,
    /// Public party registrations (party/registration.yaml).
    pub public_registrations: Mutex<HashMap<String, serde_json::Value>>,
    /// Public primary schedules (party/primaries.yaml).
    pub public_primaries: Mutex<HashMap<String, serde_json::Value>>,
    /// Public symbol registrations (symbol/registry.yaml).
    pub public_symbols: Mutex<HashMap<String, serde_json::Value>>,
    /// Initiated party document uploads by upload_ref.
    pub documents: Mutex<HashMap<String, serde_json::Value>>,
    /// Registration decisions by decision_id.
    pub registration_decisions: Mutex<HashMap<String, serde_json::Value>>,
    /// Submitted primary tallies by result id.
    ///
    /// Separate from `primary_results`, which holds the typed
    /// `crms_domain::primaries::PrimaryResult` this endpoint does not build --
    /// it accepts an opaque tally from the party portal. Shadowing the existing
    /// field would have made the domain store unreachable.
    pub submitted_primary_results: Mutex<HashMap<Uuid, serde_json::Value>>,
    /// Filed primary disputes by dispute_id.
    pub primary_disputes: Mutex<HashMap<String, serde_json::Value>>,
    /// Public nomination fees (nomination/fees.yaml).
    pub public_fees: Mutex<HashMap<String, serde_json::Value>>,
}

impl AppState {
    /// Construct an empty in-memory state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a domain event for ECC / telemetry consumers.
    pub fn record_event(&self, event: DomainEvent) {
        if let Ok(mut log) = self.events.lock() {
            log.push(event);
        }
    }
}
