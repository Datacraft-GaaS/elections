use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Emitted when a voter record is first created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterCreatedV1 {
    pub voter_id: String,
    pub county_code: String,
    pub election_scope: String,
    pub ts: DateTime<Utc>,
}

impl VoterCreatedV1 {
    pub fn event_type() -> &'static str {
        "eel.bvr.voter_created.v1"
    }
}

/// Emitted when non-PII fields of a voter record are amended.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterAmendedV1 {
    pub voter_id: String,
    pub changed_fields: Vec<String>,
    pub ts: DateTime<Utc>,
}

impl VoterAmendedV1 {
    pub fn event_type() -> &'static str {
        "eel.bvr.voter_amended.v1"
    }
}

/// Emitted when a voter record is deregistered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterDeregisteredV1 {
    pub voter_id: String,
    pub reason: String,
    pub ts: DateTime<Utc>,
}

impl VoterDeregisteredV1 {
    pub fn event_type() -> &'static str {
        "eel.bvr.voter_deregistered.v1"
    }
}

/// Emitted when the voter register is frozen ahead of an election.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterFrozenV1 {
    pub election_scope: String,
    pub frozen_at: DateTime<Utc>,
    pub authorized_by: String,
}

impl RegisterFrozenV1 {
    pub fn event_type() -> &'static str {
        "eel.bvr.register_frozen.v1"
    }
}

/// Emitted when a register snapshot is published for distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterSnapshotPublishedV1 {
    pub snapshot_id: String,
    pub election_scope: String,
    pub station_count: u32,
    pub voter_count: u64,
    pub merkle_root: String,
    pub published_at: DateTime<Utc>,
}

impl RegisterSnapshotPublishedV1 {
    pub fn event_type() -> &'static str {
        "eel.bvr.register_snapshot_published.v1"
    }
}

/// Emitted to trigger biometric deduplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupOrderedV1 {
    pub order_id: String,
    pub voter_id: String,
    pub modality: String,
    pub ts: DateTime<Utc>,
}

impl DedupOrderedV1 {
    pub fn event_type() -> &'static str {
        "eel.bvr.dedup_ordered.v1"
    }
}

/// Emitted when biometric deduplication completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupResolvedV1 {
    pub order_id: String,
    pub voter_id: String,
    pub outcome: String,
    pub ts: DateTime<Utc>,
}

impl DedupResolvedV1 {
    pub fn event_type() -> &'static str {
        "eel.bvr.dedup_resolved.v1"
    }
}

/// Emitted when a canary voter record is accessed (EC-SEC-16).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryTouchedV1 {
    pub voter_id: String,
    pub accessed_by: String,
    pub path: String,
    pub ts: DateTime<Utc>,
}

impl CanaryTouchedV1 {
    pub fn event_type() -> &'static str {
        "eel.bvr.canary_touched.v1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_types_are_namespaced() {
        assert!(VoterCreatedV1::event_type().starts_with("eel.bvr."));
        assert!(RegisterFrozenV1::event_type().starts_with("eel.bvr."));
        assert!(CanaryTouchedV1::event_type().starts_with("eel.bvr."));
    }

    #[test]
    fn voter_created_v1_roundtrip() {
        let ev = VoterCreatedV1 {
            voter_id: "v-1".into(),
            county_code: "047".into(),
            election_scope: "2027-general".into(),
            ts: Utc::now(),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: VoterCreatedV1 = serde_json::from_str(&json).unwrap();
        assert_eq!(back.voter_id, "v-1");
    }

    #[test]
    fn dedup_events_roundtrip() {
        let ordered = DedupOrderedV1 {
            order_id: "ord-1".into(),
            voter_id: "v-1".into(),
            modality: "fingerprint".into(),
            ts: Utc::now(),
        };
        let resolved = DedupResolvedV1 {
            order_id: "ord-1".into(),
            voter_id: "v-1".into(),
            outcome: "unique".into(),
            ts: Utc::now(),
        };
        let j1 = serde_json::to_string(&ordered).unwrap();
        let j2 = serde_json::to_string(&resolved).unwrap();
        assert!(j1.contains("fingerprint"));
        assert!(j2.contains("unique"));
    }
}
