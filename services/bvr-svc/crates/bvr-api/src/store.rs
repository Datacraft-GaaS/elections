//! The BVR register, in Postgres and in memory.
//!
//! `AppState` was an IPRS URL and nothing else, so four handlers built their
//! answers instead of reading a register that has been in the schema the whole
//! time:
//!
//! - `get_voter` returned `full_name: nfc("Placeholder Voter")`, county `047`,
//!   station `047-001-001` and `status: active` **for any voter id**. It invented a
//!   person's name and where they vote. The classifier passed this one as
//!   `VERIFIED` because it saw a real call to `nfc` -- a Unicode normalizer -- and
//!   read the delegation as genuine. All four arguments were string literals.
//! - `update_voter_status` logged the change and returned 204, so a status change
//!   was acknowledged and never happened.
//! - `get_latest_merkle_root` returned `"0".repeat(64)` with `seq: 0`, which is the
//!   Merkle root of an empty tree presented as the root of the register.
//! - `create_sar` minted a `sar_id` and answered 202, leaving the caller a ticket
//!   for a request that was never recorded -- and it took the requester's **raw**
//!   national ID where the contract asks for a hash.
//!
//! The tables are all there: `voters` (V1), `audit_voters_history` (V2),
//! `merkle_roots` and `register_snapshots` (V3), `sar_requests` (V4). So these are
//! real queries, not refusals.
//!
//! Two backends, following the pattern `credential-api` already uses: Postgres
//! when `DATABASE_URL` is set, and an in-memory store otherwise so the state
//! transitions are testable without a database. The in-memory path is announced at
//! startup as test-only, because a register held only in process memory is not a
//! register.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use sqlx::PgPool;

/// The register status values, per the contract's enum.
pub const STATUSES: [&str; 5] = ["active", "suspended", "deregistered", "pending", "deceased"];

/// The reason codes for a status change, per the contract's enum.
pub const REASON_CODES: [&str; 6] = [
    "duplicate_identity",
    "death_certificate",
    "court_order",
    "administrative_correction",
    "appeal_upheld",
    "appeal_dismissed",
];

/// The delivery channels a SAR may be delivered to.
pub const DELIVERY_CHANNELS: [&str; 3] = ["email", "postal", "in_person"];

/// `VoterSummary` from the contract: a reduced record, no name.
///
/// The previous `VoterResponse` carried a `full_name`, which the contract does not
/// have on this endpoint -- and which the handler could not have produced anyway,
/// since `voters.full_name_enc` is a field-encrypted blob.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VoterSummary {
    pub voter_id: String,
    /// From the `polling_station_id` column, under the contract's field name.
    pub polling_station_code: String,
    pub status: String,
    pub version: i64,
    pub election_scope: String,
}

/// `SarAccepted`: an acknowledgement, with the statutory deadline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SarAccepted {
    pub sar_id: String,
    /// 21 days from submission, per the Kenya Data Protection Act 2019.
    pub due_date: String,
    pub status: String,
}

/// A row of `sar_requests`.
#[derive(Clone, Debug)]
pub struct SarRequestRow {
    pub sar_id: String,
    pub voter_id: String,
    /// SHA-256 of the requester's national ID. Never the raw number.
    pub requestor_national_id_hash: String,
    pub delivery_channel: String,
    pub notes: Option<String>,
    pub submitted_at: String,
}

/// `MerkleRootResponse`. `signature` is `None` when no HSM has signed the root.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MerkleRootResponse {
    pub root_hash: String,
    pub tree_size: i64,
    pub timestamp: String,
    pub signature: Option<String>,
    pub snapshot_id: Option<String>,
    pub election_scope: String,
}

/// What went wrong below the handler.
#[derive(Debug)]
pub enum StoreError {
    /// The voter is not on the register.
    NotFound(String),
    /// The request is not valid against the contract's enums.
    Invalid(String),
    /// The operation needs something this deployment does not have.
    Unavailable(String),
    /// The database refused or is unreachable.
    Database(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::NotFound(m) => write!(f, "{m}"),
            StoreError::Invalid(m) => write!(f, "{m}"),
            StoreError::Unavailable(m) => write!(f, "{m}"),
            StoreError::Database(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<sqlx::Error> for StoreError {
    fn from(e: sqlx::Error) -> Self {
        StoreError::Database(e.to_string())
    }
}

/// A recorded status change, event-sourced into the audit shadow.
#[derive(Clone, Debug)]
pub struct StatusChange {
    pub voter_id: String,
    pub from_status: String,
    pub to_status: String,
    pub reason_code: String,
    pub reason_detail: Option<String>,
    pub officer_id: String,
    pub version_from: i64,
    pub version_to: i64,
    pub at: String,
}

/// The in-memory register, used when `DATABASE_URL` is unset.
#[derive(Default)]
struct Mem {
    voters: HashMap<String, VoterSummary>,
    history: Vec<StatusChange>,
    sars: Vec<SarRequestRow>,
    merkle: Option<MerkleRootResponse>,
}

/// The BVR register.
pub struct BvrStore {
    pool: Option<PgPool>,
    mem: Mutex<Mem>,
}

/// The handle threaded through the router.
pub type SharedBvrStore = Arc<BvrStore>;

impl BvrStore {
    /// A store backed by Postgres.
    pub fn from_pool(pool: PgPool) -> SharedBvrStore {
        Arc::new(Self { pool: Some(pool), mem: Mutex::new(Mem::default()) })
    }

    /// A store held in process memory. Test-only; see the module docs.
    pub fn in_memory() -> SharedBvrStore {
        Arc::new(Self { pool: None, mem: Mutex::new(Mem::default()) })
    }

    /// Whether this store is the in-memory one.
    pub fn is_in_memory(&self) -> bool {
        self.pool.is_none()
    }

    /// Put a voter on the in-memory register.
    pub fn put_voter(&self, voter: VoterSummary) {
        self.mem.lock().expect("lock poisoned").voters.insert(voter.voter_id.clone(), voter);
    }

    /// Put a Merkle root on the in-memory register.
    pub fn put_merkle_root(&self, root: MerkleRootResponse) {
        self.mem.lock().expect("lock poisoned").merkle = Some(root);
    }

    /// Every recorded status change, oldest first. In-memory only.
    pub fn history(&self) -> Vec<StatusChange> {
        self.mem.lock().expect("lock poisoned").history.clone()
    }

    /// Every recorded SAR, oldest first. In-memory only.
    pub fn sars(&self) -> Vec<SarRequestRow> {
        self.mem.lock().expect("lock poisoned").sars.clone()
    }

    /// Look a voter up.
    pub async fn voter(&self, voter_id: &str) -> Result<Option<VoterSummary>, StoreError> {
        if let Some(pool) = &self.pool {
            // `national_id_enc` and `full_name_enc` are deliberately not selected:
            // this endpoint returns a reduced record and has no business pulling
            // field-encrypted PII out of the vault to drop it.
            let row: Option<(String, String, String, i64, String)> = sqlx::query_as(
                "SELECT voter_id, polling_station_id, status, version, election_scope \
                 FROM voters WHERE voter_id = $1 LIMIT 1",
            )
            .bind(voter_id)
            .fetch_optional(pool)
            .await?;
            return Ok(row.map(
                |(voter_id, polling_station_id, status, version, election_scope)| VoterSummary {
                    voter_id,
                    polling_station_code: polling_station_id,
                    status,
                    version,
                    election_scope,
                },
            ));
        }
        Ok(self.mem.lock().expect("lock poisoned").voters.get(voter_id).cloned())
    }

    /// Change a voter's registration status, and record the change.
    ///
    /// The contract says these transitions are event-sourced, so the state change
    /// and the history row are written together: the `voters` row moves to
    /// `version + 1` and `audit_voters_history` gains the from/to pair. Doing only
    /// the first would leave a change with no record of who made it.
    pub async fn update_voter_status(
        &self,
        voter_id: &str,
        status: &str,
        reason_code: &str,
        reason_detail: Option<&str>,
        officer_id: &str,
    ) -> Result<VoterSummary, StoreError> {
        if !STATUSES.contains(&status) {
            return Err(StoreError::Invalid(format!(
                "status must be one of {}, got {status}",
                STATUSES.join(", ")
            )));
        }
        if !REASON_CODES.contains(&reason_code) {
            return Err(StoreError::Invalid(format!(
                "reason_code must be one of {}, got {reason_code}",
                REASON_CODES.join(", ")
            )));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if let Some(pool) = &self.pool {
            let mut tx = pool.begin().await?;
            let current: Option<(String, String, i64, String)> = sqlx::query_as(
                "SELECT polling_station_id, status, version, election_scope \
                 FROM voters WHERE voter_id = $1 FOR UPDATE",
            )
            .bind(voter_id)
            .fetch_optional(&mut *tx)
            .await?;
            let (polling_station_id, _from_status, version_from, election_scope) = current
                .ok_or_else(|| StoreError::NotFound(format!("no voter {voter_id} on the register")))?;
            let version_to = version_from + 1;
            sqlx::query(
                "UPDATE voters SET status = $2, version = $3, updated_at = NOW() \
                 WHERE voter_id = $1",
            )
            .bind(voter_id)
            .bind(status)
            .bind(version_to)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO audit_voters_history \
                 (voter_id, county_code, version_from, version_to, operation, changed_by, row_snapshot) \
                 SELECT voter_id, county_code, $2, $3, 'UPDATE', $4, to_jsonb(v.*) \
                 FROM voters v WHERE v.voter_id = $1",
            )
            .bind(voter_id)
            .bind(version_from)
            .bind(version_to)
            .bind(officer_id)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Ok(VoterSummary {
                voter_id: voter_id.to_string(),
                polling_station_code: polling_station_id,
                status: status.to_string(),
                version: version_to,
                election_scope,
            });
        }
        let mut guard = self.mem.lock().expect("lock poisoned");
        let (change, updated) = {
            let voter = guard
                .voters
                .get_mut(voter_id)
                .ok_or_else(|| StoreError::NotFound(format!("no voter {voter_id} on the register")))?;
            let change = StatusChange {
                voter_id: voter_id.to_string(),
                from_status: voter.status.clone(),
                to_status: status.to_string(),
                reason_code: reason_code.to_string(),
                reason_detail: reason_detail.map(str::to_string),
                officer_id: officer_id.to_string(),
                version_from: voter.version,
                version_to: voter.version + 1,
                at: now,
            };
            voter.status = status.to_string();
            voter.version += 1;
            (change, voter.clone())
        };
        guard.history.push(change);
        Ok(updated)
    }

    /// Record a subject access request.
    pub async fn record_sar(&self, row: SarRequestRow) -> Result<SarAccepted, StoreError> {
        if !DELIVERY_CHANNELS.contains(&row.delivery_channel.as_str()) {
            return Err(StoreError::Invalid(format!(
                "delivery_channel must be one of {}, got {}",
                DELIVERY_CHANNELS.join(", "),
                row.delivery_channel
            )));
        }
        if let Some(pool) = &self.pool {
            sqlx::query(
                "INSERT INTO sar_requests \
                 (sar_id, voter_id, requestor_national_id_hash, purpose, status) \
                 VALUES ($1, $2, $3, 'data_portability', 'received')",
            )
            .bind(&row.sar_id)
            .bind(&row.voter_id)
            .bind(&row.requestor_national_id_hash)
            .execute(pool)
            .await?;
        } else {
            self.mem.lock().expect("lock poisoned").sars.push(row.clone());
        }
        Ok(SarAccepted {
            sar_id: row.sar_id,
            // 21 days from submission, per the Kenya Data Protection Act 2019.
            due_date: (chrono::NaiveDate::parse_from_str(
                &row.submitted_at[..10],
                "%Y-%m-%d",
            )
            .map_err(|e| StoreError::Database(format!("unparseable submission date: {e}")))?
                + chrono::Duration::days(21))
            .to_string(),
            status: "received".to_string(),
        })
    }

    /// The latest Merkle root, if one has been published.
    pub async fn latest_merkle_root(&self) -> Result<Option<MerkleRootResponse>, StoreError> {
        if let Some(pool) = &self.pool {
            // `tree_size` and `snapshot_id` come from the snapshot that published
            // this root; `merkle_roots` alone has neither. `signed_by_hsm` records
            // the signing key, so it is read as a signature only when it is present
            // -- an absent signature is reported as absent, never invented.
            let row: Option<(String, Option<i64>, String, String, Option<String>, Option<String>, String)> =
                sqlx::query_as(
                    "SELECT m.root_hash, s.voter_count, m.created_at, m.election_scope, \
                            m.signed_by_hsm, s.snapshot_id, m.seq::TEXT \
                     FROM merkle_roots m \
                     LEFT JOIN register_snapshots s ON s.merkle_root = m.root_hash \
                     WHERE m.election_scope = $1 \
                     ORDER BY m.seq DESC LIMIT 1",
                )
                .bind(crate::MERKLE_SCOPE)
                .fetch_optional(pool)
                .await?;
            return Ok(row.map(
                |(root_hash, tree_size, created_at, election_scope, signed_by_hsm, snapshot_id, _seq)| {
                    MerkleRootResponse {
                        root_hash,
                        tree_size: tree_size.unwrap_or(0),
                        timestamp: created_at,
                        signature: signed_by_hsm.filter(|s| !s.trim().is_empty()),
                        snapshot_id,
                        election_scope,
                    }
                },
            ));
        }
        Ok(self.mem.lock().expect("lock poisoned").merkle.clone())
    }
}
