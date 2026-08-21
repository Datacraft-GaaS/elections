//! Candidate aggregate — registration, status FSM.
//! FR-M12-001 through FR-M12-010, EC-DAT-01, EC-IDN-09.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Elective positions per Kenya Elections Act Schedule 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    President,
    Governor,
    Senator,
    WomenRepresentative,
    NationalAssembly,
    CountyAssembly,
}

/// Gender for two-thirds quota rule (FR-M12-008, EC-LEG-01).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    NonBinary,
}

/// Candidate registration status lifecycle (FR-M12-012).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateStatus {
    Draft,
    Submitted,
    DocsUploaded,
    RoReviewing,
    Cleared,
    Rejected,
    Gazetted,
    Withdrawn,
}

impl Default for CandidateStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Candidate aggregate root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: Uuid,
    /// SHA-256 of national ID number (PII minimisation, FR-M12-031).
    pub id_number_hash: String,
    pub name: String,
    pub party_id: Option<Uuid>,
    pub position: Position,
    pub constituency_code: String,
    pub gender: Gender,
    /// Date of birth (ISO-8601).
    pub date_of_birth: String,
    pub is_independent: bool,
    pub status: CandidateStatus,
    pub payment_confirmed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum CandidateError {
    #[error("candidate already gazetted; withdrawal not possible")]
    AlreadyGazetted,
    #[error("nomination window closed")]
    NominationWindowClosed,
    #[error("payment not confirmed")]
    PaymentNotConfirmed,
    #[error("invalid status transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: CandidateStatus,
        to: CandidateStatus,
    },
}

impl Candidate {
    pub fn new(
        id: Uuid,
        id_number_hash: String,
        name: String,
        party_id: Option<Uuid>,
        position: Position,
        constituency_code: String,
        gender: Gender,
        date_of_birth: String,
        is_independent: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            id_number_hash,
            name,
            party_id,
            position,
            constituency_code,
            gender,
            date_of_birth,
            is_independent,
            status: CandidateStatus::Draft,
            payment_confirmed: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Transition to SUBMITTED — requires payment confirmed (FR-M47-006).
    pub fn submit(&mut self) -> Result<(), CandidateError> {
        if !self.payment_confirmed {
            return Err(CandidateError::PaymentNotConfirmed);
        }
        self.transition(CandidateStatus::Submitted)?;
        Ok(())
    }

    /// Mark docs as uploaded by candidate.
    pub fn mark_docs_uploaded(&mut self) -> Result<(), CandidateError> {
        self.transition(CandidateStatus::DocsUploaded)
    }

    /// RO begins review.
    pub fn begin_ro_review(&mut self) -> Result<(), CandidateError> {
        self.transition(CandidateStatus::RoReviewing)
    }

    /// RO clears the candidate.
    pub fn clear(&mut self) -> Result<(), CandidateError> {
        self.transition(CandidateStatus::Cleared)
    }

    /// RO rejects the candidate.
    pub fn reject(&mut self) -> Result<(), CandidateError> {
        self.transition(CandidateStatus::Rejected)
    }

    /// Gazette the candidate (dual-officer step, FR-M12-015).
    pub fn gazette(&mut self) -> Result<(), CandidateError> {
        self.transition(CandidateStatus::Gazetted)
    }

    /// Withdraw before gazette (FR-M12-014, EC-LEG-01).
    pub fn withdraw(&mut self) -> Result<(), CandidateError> {
        match &self.status {
            CandidateStatus::Gazetted => Err(CandidateError::AlreadyGazetted),
            _ => self.transition(CandidateStatus::Withdrawn),
        }
    }

    fn transition(&mut self, to: CandidateStatus) -> Result<(), CandidateError> {
        let valid = matches!(
            (&self.status, &to),
            (CandidateStatus::Draft, CandidateStatus::Submitted)
                | (CandidateStatus::Submitted, CandidateStatus::DocsUploaded)
                | (CandidateStatus::DocsUploaded, CandidateStatus::RoReviewing)
                | (CandidateStatus::RoReviewing, CandidateStatus::Cleared)
                | (CandidateStatus::RoReviewing, CandidateStatus::Rejected)
                | (CandidateStatus::Cleared, CandidateStatus::Gazetted)
                | (CandidateStatus::Submitted, CandidateStatus::Withdrawn)
                | (CandidateStatus::DocsUploaded, CandidateStatus::Withdrawn)
                | (CandidateStatus::RoReviewing, CandidateStatus::Withdrawn)
                | (CandidateStatus::Cleared, CandidateStatus::Withdrawn)
                | (CandidateStatus::Rejected, CandidateStatus::Submitted)
        );
        if !valid {
            return Err(CandidateError::InvalidTransition {
                from: self.status.clone(),
                to,
            });
        }
        self.status = to;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate() -> Candidate {
        Candidate::new(
            Uuid::new_v4(),
            "abc123hash".into(),
            "Jane Doe".into(),
            Some(Uuid::new_v4()),
            Position::NationalAssembly,
            "019".into(),
            Gender::Female,
            "1985-03-15".into(),
            false,
        )
    }

    #[test]
    fn submit_requires_payment() {
        let mut c = make_candidate();
        assert!(c.submit().is_err());
        c.payment_confirmed = true;
        assert!(c.submit().is_ok());
        assert_eq!(c.status, CandidateStatus::Submitted);
    }

    #[test]
    fn full_clearance_lifecycle() {
        let mut c = make_candidate();
        c.payment_confirmed = true;
        c.submit().unwrap();
        c.mark_docs_uploaded().unwrap();
        c.begin_ro_review().unwrap();
        c.clear().unwrap();
        c.gazette().unwrap();
        assert_eq!(c.status, CandidateStatus::Gazetted);
    }

    #[test]
    fn cannot_withdraw_after_gazette() {
        let mut c = make_candidate();
        c.payment_confirmed = true;
        c.submit().unwrap();
        c.mark_docs_uploaded().unwrap();
        c.begin_ro_review().unwrap();
        c.clear().unwrap();
        c.gazette().unwrap();
        assert!(matches!(c.withdraw(), Err(CandidateError::AlreadyGazetted)));
    }
}
