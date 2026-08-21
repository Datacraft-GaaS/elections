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
