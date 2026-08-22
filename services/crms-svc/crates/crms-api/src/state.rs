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
use uuid::Uuid;

/// Process-local registries backing the CRMS handlers.
#[derive(Default)]
pub struct AppState {
    pub candidates: Mutex<HashMap<Uuid, Candidate>>,
    pub nominations: Mutex<HashMap<Uuid, Nomination>>,
    pub clearances: Mutex<Vec<ClearanceDecision>>,
    pub gazettes: Mutex<HashMap<Uuid, Gazette>>,
    pub ballots: Mutex<HashMap<Uuid, BallotProof>>,
    pub parties: Mutex<HashMap<Uuid, PoliticalParty>>,
    pub coalitions: Mutex<HashMap<Uuid, CoalitionRegistration>>,
    pub primary_results: Mutex<HashMap<Uuid, PrimaryResult>>,
    pub disputes: Mutex<Vec<PrimaryDispute>>,
    pub fee_schedules: Mutex<Vec<FeeSchedule>>,
    pub payments: Mutex<HashMap<Uuid, FeePayment>>,
    pub symbols: Mutex<HashMap<Uuid, PartySymbol>>,
    pub events: Mutex<Vec<DomainEvent>>,
}

impl AppState {
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
