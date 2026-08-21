//! Approval policy matrix for rectification request types.

use uuid::Uuid;

use crate::domain::{EvidenceType, RectificationRequest, RectificationRequestType};
use crate::error::RectificationError;

pub struct PolicyEngine;

impl Default for PolicyEngine {
    fn default() -> Self {
        Self
    }
}

impl PolicyEngine {
    /// Returns the required evidence type for a request type, if any.
    pub fn required_evidence(&self, req_type: &RectificationRequestType) -> Option<EvidenceType> {
        match req_type {
            RectificationRequestType::BiometricRecapture => None,
            RectificationRequestType::DemographicUpdate => Some(EvidenceType::NationalId),
            RectificationRequestType::TemplateRemoval => Some(EvidenceType::OdpcReference),
            RectificationRequestType::DuplicateMerge => None,
        }
    }

    /// Returns true if an officer must be assigned before a decision.
    pub fn requires_officer(&self, req_type: &RectificationRequestType) -> bool {
        matches!(
            req_type,
            RectificationRequestType::BiometricRecapture
                | RectificationRequestType::TemplateRemoval
        )
    }

    /// Validate that a decision is permissible.
    pub fn validate_decision(
        &self,
        req: &RectificationRequest,
        officer_id: Uuid,
    ) -> Result<(), RectificationError> {
        if self.requires_officer(&req.request_type) && req.assigned_officer != Some(officer_id) {
            return Err(RectificationError::OfficerRequired);
        }
        if req.request_type == RectificationRequestType::TemplateRemoval && req.odpc_ref.is_none() {
            return Err(RectificationError::OdpcRefRequired);
        }
        Ok(())
    }
}
