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

//! Cluster resolution: merge, split, or mark as false alarm.

use chrono::Utc;

use crate::domain::{ClusterRecord, ClusterResolution};
use crate::error::ClusterError;

pub fn resolve_merge(cluster: &mut ClusterRecord) -> Result<(), ClusterError> {
    if cluster.resolution != ClusterResolution::Pending {
        return Err(ClusterError::AlreadyResolved);
    }
    cluster.resolution = ClusterResolution::MergedIntoOne;
    cluster.resolved_at = Some(Utc::now());
    Ok(())
}

pub fn resolve_split(cluster: &mut ClusterRecord) -> Result<(), ClusterError> {
    if cluster.resolution != ClusterResolution::Pending {
        return Err(ClusterError::AlreadyResolved);
    }
    cluster.resolution = ClusterResolution::SplitIntoGroups;
    cluster.resolved_at = Some(Utc::now());
    Ok(())
}

pub fn resolve_false_alarm(cluster: &mut ClusterRecord, note: &str) -> Result<(), ClusterError> {
    if cluster.resolution != ClusterResolution::Pending {
        return Err(ClusterError::AlreadyResolved);
    }
    cluster.resolution = ClusterResolution::FalseAlarm;
    cluster.resolved_at = Some(Utc::now());
    cluster.notes = Some(note.to_string());
    Ok(())
}
