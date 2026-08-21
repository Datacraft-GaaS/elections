//! Workflow enums and state machines for the five special-voter categories.
//! Elections Act ss.5, 44, 44A, 54, 72.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Special voter category (FR-M25-001 to FR-M25-029).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpecialVoterCategory {
    RemandPrison,
    Hospital,
    Barracks,
    RefugeeCamp,
    IdpCamp,
}

impl SpecialVoterCategory {
    pub fn form_variant(&self) -> &'static str {
        match self {
            Self::RemandPrison => "34A-R",
            Self::Hospital => "34A-H",
            Self::Barracks => "34A-B",
            Self::RefugeeCamp | Self::IdpCamp => "34A-I",
        }
    }

    /// Colour code for mode banner (FR-M25-048).
    pub fn banner_hex(&self) -> &'static str {
        match self {
            Self::RemandPrison => "#FF6600",
            Self::Hospital => "#0055AA",
            Self::Barracks => "#007733",
            Self::RefugeeCamp => "#884499",
            Self::IdpCamp => "#884499",
        }
    }

    /// Requires dual biometric sign-in to open kit (FR-M25-002, FR-M25-017).
    pub fn requires_dual_biometric_open(&self) -> bool {
        matches!(self, Self::RemandPrison | Self::Barracks)
    }
}

/// Kit session state for a special voter facility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FacilitySessionState {
    Sealed,
    DualBiometricPending,
    Open,
    Suspended,
    Closed,
}

impl Default for FacilitySessionState {
    fn default() -> Self {
        Self::Sealed
    }
}

/// Reason codes for special-voter events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpecialEventReasonCode {
    Standard,
    MedicalInability,
    FingerprintDegradation,
    AssistedVoting,
    SecrecyViolation,
    DoubleCastAttempted,
    S44AOverride,
}

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("FR-M25-002: dual biometric sign-in required; only one officer authenticated")]
    DualBiometricRequired,
    #[error(
        "FR-M25-039: voter {voter_id} already authenticated at IDP station; double-cast rejected"
    )]
    DoubleVoteGuard { voter_id: String },
    #[error("FR-M25-022: REFUGEE_VOTING_ENABLED flag is false; category not available")]
    RefugeeVotingDisabled,
    #[error("FR-M25-038: SECRECY_VIOLATION unacknowledged; officer acknowledgement required")]
    UnacknowledgedViolation,
}

/// Validate that a facility can open its kit session.
pub fn validate_kit_open(
    category: &SpecialVoterCategory,
    authenticated_officer_count: usize,
    refugee_voting_enabled: bool,
) -> Result<(), WorkflowError> {
    if *category == SpecialVoterCategory::RefugeeCamp && !refugee_voting_enabled {
        return Err(WorkflowError::RefugeeVotingDisabled);
    }
    if category.requires_dual_biometric_open() && authenticated_officer_count < 2 {
        return Err(WorkflowError::DualBiometricRequired);
    }
    Ok(())
}

/// Guard against IDP double-cast (FR-M25-039).
pub fn guard_idp_double_cast(
    voter_id: &str,
    already_authenticated_at_idp: bool,
) -> Result<(), WorkflowError> {
    if already_authenticated_at_idp {
        return Err(WorkflowError::DoubleVoteGuard {
            voter_id: voter_id.to_string(),
        });
    }
    Ok(())
}
