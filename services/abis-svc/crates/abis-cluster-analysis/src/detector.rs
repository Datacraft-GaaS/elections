//! Nightly job: scan recent adjudication decisions for multi-way collisions.

use chrono::{DateTime, Utc};

use crate::cluster::connected_components;
use crate::domain::{ClusterRecord, MatchEdge};
use crate::error::ClusterError;

/// Scan confirmed match edges since `since` and return duplicate clusters.
pub fn detect_clusters(
    edges: &[MatchEdge],
    since: DateTime<Utc>,
) -> Result<Vec<ClusterRecord>, ClusterError> {
    let recent: Vec<MatchEdge> = edges
        .iter()
        .filter(|e| e.confirmed_at >= since)
        .cloned()
        .collect();
    connected_components(&recent)
}
