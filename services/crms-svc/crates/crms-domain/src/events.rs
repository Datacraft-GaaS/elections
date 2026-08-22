//! Domain events published to ECC situational awareness (WP-4.11, TELE-D).
//! Handlers emit these on aggregate transitions; consumers stream them to
//! ECC dashboards and the telemetry sink.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Well-known CRMS domain event kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    CandidateSubmitted,
    CandidateCleared,
    CandidateRejected,
    CandidateGazetted,
    PartyApproved,
    PartySuspended,
    CoalitionActive,
    CoalitionDissolved,
    BallotApproved,
    GazettePublished,
    PrimaryResultSubmitted,
    QuotaViolation,
    PaymentConfirmed,
    SymbolAssigned,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::CandidateSubmitted => "candidate_submitted",
            EventType::CandidateCleared => "candidate_cleared",
            EventType::CandidateRejected => "candidate_rejected",
            EventType::CandidateGazetted => "candidate_gazetted",
            EventType::PartyApproved => "party_approved",
            EventType::PartySuspended => "party_suspended",
            EventType::CoalitionActive => "coalition_active",
            EventType::CoalitionDissolved => "coalition_dissolved",
            EventType::BallotApproved => "ballot_approved",
            EventType::GazettePublished => "gazette_published",
            EventType::PrimaryResultSubmitted => "primary_result_submitted",
            EventType::QuotaViolation => "quota_violation",
            EventType::PaymentConfirmed => "payment_confirmed",
            EventType::SymbolAssigned => "symbol_assigned",
        }
    }
}

/// An immutable, append-only domain event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: Uuid,
    pub event_type: EventType,
    /// Aggregate type name, e.g. "candidate", "party".
    pub aggregate: String,
    pub entity_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub payload: Value,
}

impl DomainEvent {
    pub fn new(
        event_type: EventType,
        aggregate: impl Into<String>,
        entity_id: Uuid,
        payload: Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            aggregate: aggregate.into(),
            entity_id,
            occurred_at: Utc::now(),
            payload,
        }
    }
}
