//! Daily cron: compute per-device Merkle root over all signals for the day,
//! submit to P05 transparency log, persist eel_telemetry_snapshots row.

use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::TelemetrySnapshot;
use crate::error::TelemetryError;

/// Build a Merkle tree from ordered leaf bytes and return the root.
///
/// Uses BLAKE3 throughout. Leaf hashing: `BLAKE3(0x00 || leaf)`.
/// Internal node hashing: `BLAKE3(0x01 || left || right)`.
/// Pads with repeated last leaf if count is odd.
pub fn compute_merkle_root(leaves: &[Vec<u8>]) -> Result<[u8; 32], TelemetryError> {
    if leaves.is_empty() {
        return Err(TelemetryError::Internal("no leaves for Merkle tree".to_owned()));
    }

    // Hash leaves.
    let mut level: Vec<[u8; 32]> = leaves
        .iter()
        .map(|leaf| {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&[0x00]);
            hasher.update(leaf);
            *hasher.finalize().as_bytes()
        })
        .collect();

    // Build tree bottom-up.
    while level.len() > 1 {
        if level.len() % 2 != 0 {
            let last = *level.last().expect("non-empty");
            level.push(last);
        }
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks(2) {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&[0x01]);
            hasher.update(&pair[0]);
            hasher.update(&pair[1]);
            next.push(*hasher.finalize().as_bytes());
        }
        level = next;
    }

    Ok(level[0])
}

/// Stub transparency log client. Production replaces with a real P05/Rekor client.
pub struct TlogClient {
    /// If true, operations succeed immediately (test mode).
    pub stub: bool,
}

impl TlogClient {
    pub fn new_stub() -> Self {
        Self { stub: true }
    }

    /// Submit a leaf to the tlog and return (leaf_index, root_hash, inclusion_proof).
    pub async fn submit(&self, _root: &[u8; 32]) -> Result<(i64, Vec<u8>, Vec<u8>), TelemetryError> {
        if self.stub {
            // Return deterministic stub values for tests.
            let leaf_index = 42i64;
            let root_hash = vec![0xAB; 32];
            let inclusion_proof = vec![0xCD; 64];
            return Ok((leaf_index, root_hash, inclusion_proof));
        }
        Err(TelemetryError::TlogSubmit("real tlog not configured".to_owned()))
    }
}

/// Compute the Merkle snapshot for `device_id` on `date` from signal hashes.
///
/// `signal_hashes` must be ordered by monotonic_counter ascending.
pub async fn take_snapshot(
    device_id: Uuid,
    date: NaiveDate,
    signal_hashes: Vec<Vec<u8>>,
    tlog: &TlogClient,
) -> Result<TelemetrySnapshot, TelemetryError> {
    let merkle_root = compute_merkle_root(&signal_hashes)?;
    let (leaf_index, tlog_root, proof) = tlog.submit(&merkle_root).await?;

    Ok(TelemetrySnapshot {
        snapshot_id: Uuid::now_v7(),
        snapshot_date: date,
        device_id: Some(device_id),
        device_merkle_root: Some(merkle_root.to_vec()),
        snapshot_taken_at: Utc::now(),
        tlog_leaf_index: Some(leaf_index),
        tlog_root_hash: Some(tlog_root),
        inclusion_proof: Some(proof),
    })
}
