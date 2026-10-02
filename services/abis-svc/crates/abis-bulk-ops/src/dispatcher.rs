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

//! Dispatcher that routes bulk operations to the appropriate worker.

use crate::domain::{BulkOpState, BulkOperation};
use crate::error::BulkOpsError;
use chrono::Utc;
use tracing::info;

/// Validate that an operation can be started or resumed.
pub fn validate_runnable(op: &BulkOperation) -> Result<(), BulkOpsError> {
    match op.state {
        BulkOpState::Pending | BulkOpState::Paused | BulkOpState::Running => Ok(()),
        _ => Err(BulkOpsError::AlreadyTerminal),
    }
}

/// Transition an operation to Running state.
pub fn mark_running(op: &mut BulkOperation) {
    op.state = BulkOpState::Running;
    if op.started_at.is_none() {
        op.started_at = Some(Utc::now());
    }
    info!(op_id = %op.id, op_type = %op.op_type, "bulk op running");
}

/// Pause a running operation.
pub fn pause(op: &mut BulkOperation) -> Result<(), BulkOpsError> {
    if op.state != BulkOpState::Running {
        return Err(BulkOpsError::NotResumable);
    }
    op.state = BulkOpState::Paused;
    Ok(())
}

/// Cancel an operation.
pub fn cancel(op: &mut BulkOperation) -> Result<(), BulkOpsError> {
    match op.state {
        BulkOpState::Completed | BulkOpState::Failed | BulkOpState::Cancelled => {
            Err(BulkOpsError::AlreadyTerminal)
        }
        _ => {
            op.state = BulkOpState::Cancelled;
            Ok(())
        }
    }
}
