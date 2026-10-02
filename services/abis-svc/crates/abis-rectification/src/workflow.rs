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

//! State machine transitions for rectification workflow.

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::domain::{RectificationRequest, RectificationState};
use crate::error::RectificationError;
use crate::policy::PolicyEngine;

/// Transition a request from Submitted to Triage.
pub fn to_triage(req: &mut RectificationRequest) -> Result<(), RectificationError> {
    if req.state != RectificationState::Submitted {
        return Err(RectificationError::InvalidTransition {
            from: req.state.clone(),
            to: RectificationState::Triage,
        });
    }
    req.state = RectificationState::Triage;
    info!(req_id = %req.id, "rectification moved to triage");
    Ok(())
}

/// Transition from Triage to EvidencePending.
pub fn to_evidence_pending(req: &mut RectificationRequest) -> Result<(), RectificationError> {
    let _policy = PolicyEngine::default();
    if req.state != RectificationState::Triage {
        return Err(RectificationError::InvalidTransition {
            from: req.state.clone(),
            to: RectificationState::EvidencePending,
        });
    }
    req.state = RectificationState::EvidencePending;
    Ok(())
}

/// Assign an officer to a request.
pub fn assign_officer(
    req: &mut RectificationRequest,
    officer_id: Uuid,
) -> Result<(), RectificationError> {
    match req.state {
        RectificationState::Triage | RectificationState::EvidencePending => {
            req.assigned_officer = Some(officer_id);
            Ok(())
        }
        _ => Err(RectificationError::InvalidTransition {
            from: req.state.clone(),
            to: req.state.clone(),
        }),
    }
}

/// Record a decision (Approved or Rejected).
pub fn decide(
    req: &mut RectificationRequest,
    approved: bool,
    decision_note: &str,
    officer_id: Uuid,
) -> Result<(), RectificationError> {
    let policy = PolicyEngine::default();
    policy.validate_decision(req, officer_id)?;
    match req.state {
        RectificationState::Triage | RectificationState::EvidencePending => {}
        _ => {
            return Err(RectificationError::InvalidTransition {
                from: req.state.clone(),
                to: if approved {
                    RectificationState::Approved
                } else {
                    RectificationState::Rejected
                },
            });
        }
    }
    req.state = if approved {
        RectificationState::Approved
    } else {
        RectificationState::Rejected
    };
    req.decision = Some(decision_note.to_string());
    req.decision_at = Some(Utc::now());
    info!(req_id = %req.id, approved, "rectification decided");
    Ok(())
}

/// Mark a request as completed.
pub fn complete(req: &mut RectificationRequest) -> Result<(), RectificationError> {
    if req.state != RectificationState::Approved {
        return Err(RectificationError::InvalidTransition {
            from: req.state.clone(),
            to: RectificationState::Completed,
        });
    }
    req.state = RectificationState::Completed;
    req.completed_at = Some(Utc::now());
    info!(req_id = %req.id, "rectification completed");
    Ok(())
}
