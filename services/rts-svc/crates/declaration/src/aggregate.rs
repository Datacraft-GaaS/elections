//! Event-sourced Declaration, CertificateOfElection, and PetitionWindowTracker
//! aggregates (WP-3.3, M51).

use chrono::Utc;
use eel_eventstore_rs::{Aggregate, AggregateSnapshot, EventStoreError, StoredEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::*;

// ---- Declaration ------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeclarationStatus {
    Pending,
    Declared,
    RunoffRequired,
    TiePending,
    Amended,
    Annulled,
}

impl Default for DeclarationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Declaration {
    pub stream_id: Uuid,
    pub version: i64,
    pub contest_id: String,
    pub election_id: String,
    pub status: DeclarationStatus,
    pub winner_id: Option<String>,
    pub declared_at: Option<String>,
    pub declared_by: Option<String>,
    pub ceremony_id: Option<String>,
    pub hsm_signature: Option<String>,
    pub tally_snapshot_id: Option<String>,
    /// Lineage of amendment ids (append-only; no deletion).
    pub amendment_ids: Vec<String>,
    pub runoff_candidates: Vec<String>,
    pub tied_candidates: Vec<String>,
    pub recount_court_ref: Option<String>,
}

impl Declaration {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            contest_id: String::new(),
            election_id: String::new(),
            status: DeclarationStatus::Pending,
            winner_id: None,
            declared_at: None,
            declared_by: None,
            ceremony_id: None,
            hsm_signature: None,
            tally_snapshot_id: None,
            amendment_ids: Vec::new(),
            runoff_candidates: Vec::new(),
            tied_candidates: Vec::new(),
            recount_court_ref: None,
        }
    }
}

impl Aggregate for Declaration {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        match event.event_type.as_str() {
            EVENT_RESULT_DECLARED => {
                self.status = DeclarationStatus::Declared;
                if let Some(v) = event.payload.get("contest_id").and_then(|v| v.as_str()) {
                    self.contest_id = v.to_string();
                }
                if let Some(v) = event.payload.get("election_id").and_then(|v| v.as_str()) {
                    self.election_id = v.to_string();
                }
                if let Some(v) = event.payload.get("winner_id").and_then(|v| v.as_str()) {
                    self.winner_id = Some(v.to_string());
                }
                if let Some(v) = event.payload.get("declared_at").and_then(|v| v.as_str()) {
                    self.declared_at = Some(v.to_string());
                }
                if let Some(v) = event.payload.get("declared_by").and_then(|v| v.as_str()) {
                    self.declared_by = Some(v.to_string());
                }
                if let Some(v) = event.payload.get("ceremony_id").and_then(|v| v.as_str()) {
                    self.ceremony_id = Some(v.to_string());
                }
                if let Some(v) = event.payload.get("hsm_signature").and_then(|v| v.as_str()) {
                    self.hsm_signature = Some(v.to_string());
                }
                if let Some(v) = event
                    .payload
                    .get("tally_snapshot_id")
                    .and_then(|v| v.as_str())
                {
                    self.tally_snapshot_id = Some(v.to_string());
                }
            }
            EVENT_RESULT_AMENDED => {
                self.status = DeclarationStatus::Amended;
                if let Some(v) = event.payload.get("amendment_id").and_then(|v| v.as_str()) {
                    self.amendment_ids.push(v.to_string());
                }
                if let Some(v) = event
                    .payload
                    .get("court_order_ref")
                    .and_then(|v| v.as_str())
                {
                    self.recount_court_ref = Some(v.to_string());
                }
                // Re-declaration after amendment transitions back to Declared.
                // The subsequent re-declare event will move the status.
            }
            EVENT_RECOUNT_ORDERED => {
                if let Some(v) = event
                    .payload
                    .get("court_order_ref")
                    .and_then(|v| v.as_str())
                {
                    self.recount_court_ref = Some(v.to_string());
                }
            }
            EVENT_RUNOFF_REQUIRED => {
                self.status = DeclarationStatus::RunoffRequired;
                if let Some(arr) = event.payload.get("runoff_candidates") {
                    if let Ok(v) = serde_json::from_value::<Vec<String>>(arr.clone()) {
                        self.runoff_candidates = v;
                    }
                }
            }
            EVENT_TIE_PENDING => {
                self.status = DeclarationStatus::TiePending;
                if let Some(arr) = event.payload.get("tied_candidates") {
                    if let Ok(v) = serde_json::from_value::<Vec<String>>(arr.clone()) {
                        self.tied_candidates = v;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn snapshot(&self) -> AggregateSnapshot {
        AggregateSnapshot {
            stream_id: self.stream_id,
            stream_version: self.version,
            state: serde_json::to_value(self).unwrap_or_default(),
            taken_at: Utc::now(),
        }
    }

    fn stream_id(&self) -> Uuid {
        self.stream_id
    }
}

// ---- CertificateOfElection --------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateStatus {
    Draft,
    Issued,
    Superseded,
    Revoked,
}

impl Default for CertificateStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateOfElection {
    pub stream_id: Uuid,
    pub version: i64,
    pub declaration_id: String,
    pub winner_id: String,
    pub status: CertificateStatus,
    pub issued_at: Option<String>,
    pub hsm_signature: Option<String>,
    pub pdf_minio_ref: Option<String>,
    pub tlog_leaf_index: Option<i64>,
}

impl CertificateOfElection {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            declaration_id: String::new(),
            winner_id: String::new(),
            status: CertificateStatus::Draft,
            issued_at: None,
            hsm_signature: None,
            pdf_minio_ref: None,
            tlog_leaf_index: None,
        }
    }
}

impl Aggregate for CertificateOfElection {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        match event.event_type.as_str() {
            EVENT_CERTIFICATE_ISSUED => {
                self.status = CertificateStatus::Issued;
                if let Some(v) = event.payload.get("declaration_id").and_then(|v| v.as_str()) {
                    self.declaration_id = v.to_string();
                }
                if let Some(v) = event.payload.get("winner_id").and_then(|v| v.as_str()) {
                    self.winner_id = v.to_string();
                }
                if let Some(v) = event.payload.get("issued_at").and_then(|v| v.as_str()) {
                    self.issued_at = Some(v.to_string());
                }
                if let Some(v) = event.payload.get("hsm_signature").and_then(|v| v.as_str()) {
                    self.hsm_signature = Some(v.to_string());
                }
                if let Some(v) = event.payload.get("pdf_minio_ref").and_then(|v| v.as_str()) {
                    self.pdf_minio_ref = Some(v.to_string());
                }
                if let Some(v) = event
                    .payload
                    .get("tlog_leaf_index")
                    .and_then(|v| v.as_i64())
                {
                    self.tlog_leaf_index = Some(v);
                }
            }
            EVENT_RESULT_AMENDED => {
                // Certificate superseded on amendment (FR-M51-041).
                if self.status == CertificateStatus::Issued {
                    self.status = CertificateStatus::Superseded;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn snapshot(&self) -> AggregateSnapshot {
        AggregateSnapshot {
            stream_id: self.stream_id,
            stream_version: self.version,
            state: serde_json::to_value(self).unwrap_or_default(),
            taken_at: Utc::now(),
        }
    }

    fn stream_id(&self) -> Uuid {
        self.stream_id
    }
}

// ---- PetitionWindowTracker --------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PetitionWindowStatus {
    Unopened,
    Open,
    Closed,
    Petitioned,
}

impl Default for PetitionWindowStatus {
    fn default() -> Self {
        Self::Unopened
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetitionWindowTracker {
    pub stream_id: Uuid,
    pub version: i64,
    pub declaration_id: String,
    pub status: PetitionWindowStatus,
    pub window_open_at: Option<String>,
    pub window_close_at: Option<String>,
    pub petition_court_case: Option<String>,
    pub judiciary_ack_at: Option<String>,
}

impl PetitionWindowTracker {
    pub fn new(stream_id: Uuid) -> Self {
        Self {
            stream_id,
            version: 0,
            declaration_id: String::new(),
            status: PetitionWindowStatus::Unopened,
            window_open_at: None,
            window_close_at: None,
            petition_court_case: None,
            judiciary_ack_at: None,
        }
    }
}

impl Aggregate for PetitionWindowTracker {
    fn apply(&mut self, event: &StoredEvent) -> Result<(), EventStoreError> {
        self.version = event.stream_version;
        match event.event_type.as_str() {
            EVENT_PETITION_WINDOW_OPENED => {
                self.status = PetitionWindowStatus::Open;
                if let Some(v) = event.payload.get("declaration_id").and_then(|v| v.as_str()) {
                    self.declaration_id = v.to_string();
                }
                if let Some(v) = event.payload.get("window_open_at").and_then(|v| v.as_str()) {
                    self.window_open_at = Some(v.to_string());
                }
                if let Some(v) = event
                    .payload
                    .get("window_close_at")
                    .and_then(|v| v.as_str())
                {
                    self.window_close_at = Some(v.to_string());
                }
            }
            EVENT_PETITION_FILED => {
                self.status = PetitionWindowStatus::Petitioned;
                if let Some(v) = event
                    .payload
                    .get("court_case_number")
                    .and_then(|v| v.as_str())
                {
                    self.petition_court_case = Some(v.to_string());
                }
            }
            EVENT_JUDICIARY_ACK => {
                if let Some(v) = event
                    .payload
                    .get("acknowledged_at")
                    .and_then(|v| v.as_str())
                {
                    self.judiciary_ack_at = Some(v.to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn snapshot(&self) -> AggregateSnapshot {
        AggregateSnapshot {
            stream_id: self.stream_id,
            stream_version: self.version,
            state: serde_json::to_value(self).unwrap_or_default(),
            taken_at: Utc::now(),
        }
    }

    fn stream_id(&self) -> Uuid {
        self.stream_id
    }
}

// ---- Invariant checks (used in tests and kani proofs) -----------------------

impl Declaration {
    /// A certificate must never be issued when status is TiePending (EC-LEG-06).
    pub fn may_issue_certificate(&self) -> bool {
        self.status == DeclarationStatus::Declared
    }

    /// Amendment must preserve the original event stream (append-only, EC-LEG-11).
    pub fn amendment_count(&self) -> usize {
        self.amendment_ids.len()
    }
}
