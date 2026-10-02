//! In-memory store for the RTS HTTP surface.
//!
//! Every handler in this crate answered from constants and `Vec::new()`:
//!
//! - `get_form_proof` returned `merkle_root: "0".repeat(64)` — **the zero root**,
//!   which is not a Merkle root of anything. Any client verifying against it
//!   would compute a different root and (correctly) reject, but a client that
//!   only checked "is a root present" would accept it. `packages/eel-crypto-rs`
//!   has a real `MerkleTree` with `inclusion_proof` and
//!   `verify_inclusion_against_root`, and it was not used here.
//! - `get_snapshot` returned `merkle_root: None` and `form34c: None` for any
//!   scope/code, and `list_snapshots` returned an empty list.
//! - `get_ack` returned `status: "pending", form_id: None` for **any** scan id,
//!   so a form that had been submitted and scanned read as never received.
//! - `get_icr_job` minted a **new** `job_id` on every read, so polling an ICR
//!   job for a form produced a different id each time — a client could never
//!   correlate two polls.
//! - `get_icr_result` answered `pending` with no fields, for any job id, forever.
//! - `provision_token` returned `rts-token-stub-<uuid>` with a hardcoded
//!   `expires_at` of `2030-01-01T00:00:00Z`.
//! - `resolve_dispute` answered `resolved` for any dispute id, and never checked
//!   the `countersign_token` the request carries.
//! - `compile_form34b` / `compile_form34c` answered `compilation_queued` for any
//!   input and recorded no job.
//!
//! The Merkle tree here is built over the Form 34A submissions **this process has
//! received**, in arrival order. That is a real root and a real inclusion proof,
//! and it is verifiable with `eel_crypto_rs`. It is *not* a transparency-log
//! inclusion proof: there is no append-only log in this workspace to submit a
//! leaf to, and no cross-process consensus over the leaf set. That is stated on
//! the response rather than left for a reader to assume.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use uuid::Uuid;

use crate::models::*;

/// A received Form 34A, as stored.
#[derive(Clone, Debug)]
pub struct StoredForm {
    /// The form's own id.
    pub form_id: Uuid,
    /// The submission as received.
    pub request: Form34ARequest,
    /// The station the form came from.
    pub station_code: String,
    /// The constituency the form belongs to.
    pub constituency_code: String,
    /// The county the form belongs to.
    pub county_code: String,
    /// Position in the arrival-ordered leaf set, assigned at receipt.
    pub leaf_index: usize,
    /// The leaf hash: a domain-separated digest over the canonical form.
    pub leaf_hash: String,
    /// Whether the form arrived over the SMS fallback path.
    pub via_sms: bool,
    pub received_at: String,
    /// ICR extraction result, once one exists.
    pub icr: Option<IcrJobResult>,
    /// Disputes raised against this form.
    pub disputes: Vec<DisputeCreated>,
}

/// A provisioned device token.
#[derive(Clone, Debug)]
pub struct StoredToken {
    pub device_id: String,
    pub token: String,
    pub issued_at: String,
    pub expires_at: String,
}

/// An ingested event envelope.
#[derive(Clone, Debug)]
pub struct StoredEvent {
    pub event_id: String,
    pub envelope: Value,
    pub received_at: String,
}

/// A Form 34B / 34C compilation job.
#[derive(Clone, Debug)]
pub struct CompilationJob {
    pub job_id: String,
    pub form_kind: String,
    pub status: String,
    pub requested_at: String,
    /// Why the job cannot progress on its own.
    pub blocking_reason: Option<String>,
}

/// An open or resolved dispute.
#[derive(Clone, Debug)]
pub struct StoredDispute {
    pub dispute: DisputeCreated,
    pub form_id: Uuid,
    pub reason: String,
    pub ro_id: String,
    pub resolution: Option<String>,
    pub resolved_at: Option<String>,
}

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    /// Forms in arrival order. The position in this list is the leaf index.
    forms: Mutex<Vec<StoredForm>>,
    by_form: Mutex<HashMap<Uuid, usize>>,
    /// Acknowledgements by `scan_id`.
    acks: Mutex<HashMap<Uuid, Uuid>>,
    tokens: Mutex<Vec<StoredToken>>,
    events: Mutex<Vec<StoredEvent>>,
    jobs: Mutex<Vec<CompilationJob>>,
    disputes: Mutex<HashMap<Uuid, StoredDispute>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

/// The domain separator for a Form 34A leaf.
const FORM34A_LEAF_DOMAIN: &[u8] = b"eel.rts.form34a.v1";

impl AppState {
    /// Construct an empty store.
    pub fn new() -> SharedState {
        Arc::new(Self::default())
    }

    /// The leaf hash for a form: a domain-separated digest over the canonical
    /// request, so a form cannot be replayed as a different kind of record.
    pub fn leaf_hash(request: &Form34ARequest) -> String {
        use sha2::Digest;
        let canonical = serde_json::to_vec(request).expect("Form34ARequest is JSON-serialisable");
        let mut h = sha2::Sha256::new();
        h.update(FORM34A_LEAF_DOMAIN);
        h.update([0x1e]);
        h.update(&canonical);
        format!("sha256:{}", hex::encode(h.finalize()))
    }

    /// Store a received form and return its record.
    pub fn record_form(&self, request: Form34ARequest, via_sms: bool) -> StoredForm {
        let mut guard = self.forms.lock().expect("lock poisoned");
        let leaf_index = guard.len();
        let stored = StoredForm {
            form_id: Uuid::now_v7(),
            station_code: request.station_code.clone(),
            constituency_code: request.constituency_code.clone(),
            county_code: request.county_code.clone(),
            leaf_hash: Self::leaf_hash(&request),
            request,
            leaf_index,
            via_sms,
            received_at: chrono::Utc::now().to_rfc3339(),
            icr: None,
            disputes: Vec::new(),
        };
        guard.push(stored.clone());
        drop(guard);
        self.by_form
            .lock()
            .expect("lock poisoned")
            .insert(stored.form_id, stored.leaf_index);
        stored
    }

    /// A form by id.
    pub fn form(&self, form_id: &Uuid) -> Option<StoredForm> {
        let index = *self.by_form.lock().expect("lock poisoned").get(form_id)?;
        self.forms.lock().expect("lock poisoned").get(index).cloned()
    }

    /// Every form, in arrival order.
    pub fn forms(&self) -> Vec<StoredForm> {
        self.forms.lock().expect("lock poisoned").clone()
    }

    /// The Merkle tree over every received form, in arrival order.
    ///
    /// Returns `None` when no form has been received: an empty tree has no root
    /// worth publishing, and a root over nothing is the zero-root problem again.
    pub fn merkle(&self) -> Option<eel_crypto_rs::merkle::MerkleTree> {
        let forms = self.forms();
        if forms.is_empty() {
            return None;
        }
        let entries: Vec<&[u8]> = forms.iter().map(|f| f.leaf_hash.as_bytes()).collect();
        Some(eel_crypto_rs::merkle::MerkleTree::build(&entries))
    }

    /// Record that a scan was received for a form.
    pub fn acknowledge(&self, scan_id: Uuid, form_id: Uuid) {
        self.acks
            .lock()
            .expect("lock poisoned")
            .insert(scan_id, form_id);
    }

    /// The form a scan was acknowledged for.
    pub fn acknowledged(&self, scan_id: &Uuid) -> Option<Uuid> {
        self.acks.lock().expect("lock poisoned").get(scan_id).copied()
    }

    /// Store a token.
    pub fn record_token(&self, token: StoredToken) {
        self.tokens.lock().expect("lock poisoned").push(token);
    }

    /// A token by its value.
    pub fn token(&self, value: &str) -> Option<StoredToken> {
        self.tokens
            .lock()
            .expect("lock poisoned")
            .iter()
            .find(|t| t.token == value)
            .cloned()
    }

    /// Record an ingested event.
    pub fn record_event(&self, envelope: Value) -> StoredEvent {
        let stored = StoredEvent {
            event_id: Uuid::now_v7().to_string(),
            envelope,
            received_at: chrono::Utc::now().to_rfc3339(),
        };
        self.events.lock().expect("lock poisoned").push(stored.clone());
        stored
    }

    /// Every ingested event.
    pub fn events(&self) -> Vec<StoredEvent> {
        self.events.lock().expect("lock poisoned").clone()
    }

    /// Record a compilation job.
    pub fn record_job(&self, job: CompilationJob) {
        self.jobs.lock().expect("lock poisoned").push(job);
    }

    /// Every compilation job.
    pub fn jobs(&self) -> Vec<CompilationJob> {
        self.jobs.lock().expect("lock poisoned").clone()
    }

    /// Store an ICR result against a form.
    pub fn set_icr(&self, form_id: &Uuid, result: IcrJobResult) {
        if let Some(index) = self.by_form.lock().expect("lock poisoned").get(form_id) {
            let mut guard = self.forms.lock().expect("lock poisoned");
            if let Some(form) = guard.get_mut(*index) {
                form.icr = Some(result);
            }
        }
    }

    /// Record a dispute.
    pub fn record_dispute(&self, dispute: StoredDispute) {
        let dispute_id = dispute.dispute.dispute_id;
        if let Some(index) = self.by_form.lock().expect("lock poisoned").get(&dispute.form_id) {
            let mut guard = self.forms.lock().expect("lock poisoned");
            if let Some(form) = guard.get_mut(*index) {
                form.disputes.push(dispute.dispute.clone());
            }
        }
        self.disputes
            .lock()
            .expect("lock poisoned")
            .insert(dispute_id, dispute);
    }

    /// A dispute by id.
    pub fn dispute(&self, dispute_id: &Uuid) -> Option<StoredDispute> {
        self.disputes.lock().expect("lock poisoned").get(dispute_id).cloned()
    }

    /// Record a resolution against a dispute, returning it if one existed.
    pub fn resolve_dispute(
        &self,
        dispute_id: &Uuid,
        resolution: &str,
    ) -> Option<StoredDispute> {
        let mut guard = self.disputes.lock().expect("lock poisoned");
        let d = guard.get_mut(dispute_id)?;
        d.resolution = Some(resolution.to_string());
        d.resolved_at = Some(chrono::Utc::now().to_rfc3339());
        d.dispute.status = "resolved".to_string();
        Some(d.clone())
    }

    /// The consolidated Form 34C for a constituency.
    ///
    /// This is the actual the Commission consolidation: a candidate's votes across a
    /// constituency are the sum over every station that reported them, and a
    /// candidate is only included if at least one station returned a figure for
    /// them. Previously `get_snapshot` returned `form34c: None` for every scope.
    pub fn consolidated_34c(&self, constituency_code: &str) -> Option<Form34C> {
        let mut totals: HashMap<String, (String, u64)> = HashMap::new();
        let mut registered_voters = 0u32;
        let mut votes_cast = 0u32;
        let mut valid_votes = 0u32;
        let mut rejected_votes = 0u32;
        let mut spoilt_ballots = 0u32;
        let mut stations: Vec<String> = Vec::new();
        let mut saw_any = false;
        for form in self.forms() {
            if form.constituency_code != constituency_code {
                continue;
            }
            saw_any = true;
            if !stations.contains(&form.station_code) {
                stations.push(form.station_code.clone());
            }
            registered_voters += form.request.registered_voters;
            votes_cast += form.request.votes_cast;
            valid_votes += form.request.valid_votes;
            rejected_votes += form.request.rejected_votes;
            spoilt_ballots += form.request.spoilt_ballots;
            for c in &form.request.candidates {
                let entry = totals
                    .entry(c.candidate_id.clone())
                    .or_insert_with(|| (c.candidate_name.clone(), 0));
                entry.1 += c.votes as u64;
            }
        }
        if !saw_any {
            return None;
        }
        let mut candidates: Vec<Form34CCandidate> = totals
            .into_iter()
            .map(|(candidate_id, (candidate_name, votes))| Form34CCandidate {
                candidate_id,
                candidate_name,
                votes,
            })
            .collect();
        // Deterministic order so two reads of the same data agree.
        candidates.sort_by(|a, b| b.votes.cmp(&a.votes).then(a.candidate_id.cmp(&b.candidate_id)));
        Some(Form34C {
            constituency_code: constituency_code.to_string(),
            stations_reporting: stations,
            registered_voters,
            votes_cast,
            valid_votes,
            rejected_votes,
            spoilt_ballots,
            candidates,
        })
    }
}

/// A consolidated Form 34C.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Form34C {
    pub constituency_code: String,
    pub stations_reporting: Vec<String>,
    pub registered_voters: u32,
    pub votes_cast: u32,
    pub valid_votes: u32,
    pub rejected_votes: u32,
    pub spoilt_ballots: u32,
    pub candidates: Vec<Form34CCandidate>,
}

/// One candidate's consolidated total.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Form34CCandidate {
    pub candidate_id: String,
    pub candidate_name: String,
    pub votes: u64,
}
