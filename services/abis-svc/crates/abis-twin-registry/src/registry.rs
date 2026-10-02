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

//! Twin group management operations.

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::domain::{TwinGroup, TwinMember, TwinVerificationStatus};
use crate::error::TwinRegistryError;

/// Create a new twin group with initial members.
pub fn create_twin_group(member_ids: &[Uuid]) -> Result<TwinGroup, TwinRegistryError> {
    if member_ids.len() < 2 {
        return Err(TwinRegistryError::InsufficientMembers);
    }
    let mut group = TwinGroup::new();
    for &id in member_ids {
        group.members.push(TwinMember {
            subject_id: id,
            joined_at: Utc::now(),
            civil_registry_id: None,
        });
    }
    info!(group_id = %group.id, member_count = group.members.len(), "twin group created");
    Ok(group)
}

/// Add a member to an existing twin group.
pub fn add_member(group: &mut TwinGroup, subject_id: Uuid) -> Result<(), TwinRegistryError> {
    if group.members.iter().any(|m| m.subject_id == subject_id) {
        return Err(TwinRegistryError::AlreadyInGroup(subject_id));
    }
    group.members.push(TwinMember {
        subject_id,
        joined_at: Utc::now(),
        civil_registry_id: None,
    });
    Ok(())
}

/// Simulate verification from civil registry.
/// When all members have a civil_registry_id, status transitions to VerifiedByCivilRegistry.
pub fn verify_from_civil_registry(
    group: &mut TwinGroup,
    civil_ids: &[(Uuid, String)],
) -> Result<(), TwinRegistryError> {
    for (subject_id, civil_id) in civil_ids {
        match group
            .members
            .iter_mut()
            .find(|m| m.subject_id == *subject_id)
        {
            Some(m) => m.civil_registry_id = Some(civil_id.clone()),
            None => return Err(TwinRegistryError::NotFound(*subject_id)),
        }
    }
    if group.members.iter().all(|m| m.civil_registry_id.is_some()) {
        group.verification_status = TwinVerificationStatus::VerifiedByCivilRegistry;
        group.verified_at = Some(Utc::now());
        info!(group_id = %group.id, "twin group verified by civil registry");
    } else {
        group.verification_status = TwinVerificationStatus::PendingCivilRegistry;
    }
    Ok(())
}
