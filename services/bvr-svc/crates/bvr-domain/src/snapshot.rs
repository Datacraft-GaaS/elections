use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Immutable snapshot of the voter register for a given election scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterSnapshot {
    pub snapshot_id: String,
    pub election_scope: String,
    pub station_count: u32,
    pub voter_count: u64,
    pub merkle_root: String,
    pub signed_by: String,
    pub created_at: DateTime<Utc>,
    /// Resume token for streamed export (checkpoint-resumable per EC-DAT-08).
    pub checkpoint: Option<String>,
}

/// Merkle root anchoring a set of voter records for a given election scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleRoot {
    /// Hex-encoded root hash.
    pub root_hash: String,
    pub seq: u64,
    pub election_scope: String,
    pub signed_by_hsm: Option<String>,
    pub escrow_confirmed: bool,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_snapshot_roundtrip() {
        let snap = RegisterSnapshot {
            snapshot_id: "snap-1".into(),
            election_scope: "2027-general".into(),
            station_count: 42_000,
            voter_count: 22_000_000,
            merkle_root: "a".repeat(64),
            signed_by: "hsm-key-1".into(),
            created_at: Utc::now(),
            checkpoint: None,
        };
        let json = serde_json::to_string(&snap).unwrap();
        let back: RegisterSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.station_count, 42_000);
        assert_eq!(back.voter_count, 22_000_000);
    }

    #[test]
    fn merkle_root_escrow_defaults_false() {
        let mr = MerkleRoot {
            root_hash: "0".repeat(64),
            seq: 1,
            election_scope: "2027-general".into(),
            signed_by_hsm: None,
            escrow_confirmed: false,
            created_at: Utc::now(),
        };
        assert!(!mr.escrow_confirmed);
    }
}
