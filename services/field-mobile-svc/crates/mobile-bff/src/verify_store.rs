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

//! In-memory state for the public `/verify/v1/*` surface.
//!
//! Every handler on that surface answered with a constant or a null:
//!
//! - `verify_token` returned **`included: true` for any token at all**, with a
//!   proof reference that was just `proof-<query_id>`. This is the endpoint a
//!   member of the public uses to check whether their token is in the published
//!   results, and it said yes to everything. Nothing in this workspace records
//!   published results, so a token cannot honestly be *included*; what can be
//!   answered is whether it was ever recorded here, which is a different question
//!   and is now the one asked.
//! - `get_token_proof` returned `"proof": null` with a note, for any sha.
//! - `get_results` returned `data: []` and `get_results_by_scope` returned
//!   `data: null`, both described as a "relay from M11" — a service not present in
//!   this workspace.
//! - `get_queue_snapshot` returned `queue_length: 0` for any station, which reads
//!   as "no one is waiting".
//! - `get_anomalies` returned `anomalies: []` for any query, which reads as "no
//!   anomalies detected".
//!
//! What this store holds is real: the tokens recorded through this BFF, a Merkle
//! tree over them built with `eel_crypto_rs`, the per-station queue state agents
//! check in against, the anomaly flags derived from that queue state, and the
//! clone reports filed through the public form.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use eel_crypto_rs::merkle::MerkleTree;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// A recorded token, with the leaf it contributes to the tree.
#[derive(Clone, Debug)]
pub struct TokenRecord {
    /// The token's SHA-256, which is the only form this service holds.
    pub token_sha256: String,
    /// The constituency the token was issued for, when known.
    pub constituency_code: Option<String>,
    /// Where it was recorded.
    pub source: String,
    /// Its position in the arrival-ordered leaf set.
    pub leaf_index: usize,
    /// The domain-separated leaf hash.
    pub leaf_hash: String,
    pub recorded_at: String,
}

/// A station queue, as agents check in and out against it.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct StationQueue {
    /// Agents currently checked in, by their hashed identity.
    pub checked_in: Vec<String>,
    /// Every check-in this station has seen, in order.
    pub history: Vec<String>,
}

/// A public anomaly flag.
#[derive(Clone, Debug, serde::Serialize)]
pub struct AnomalyFlag {
    pub anomaly_id: String,
    pub kind: String,
    pub station: String,
    pub detail: String,
    pub raised_at: String,
}

/// A filed clone-detection report.
#[derive(Clone, Debug, serde::Serialize)]
pub struct CloneReport {
    pub report_id: String,
    pub package_name: String,
    pub build_id: String,
    pub findings: Vec<String>,
    /// The digest of the report, so two identical reports are distinguishable.
    pub report_hash: String,
    pub filed_at: String,
}

/// Shared state for the verify surface.
#[derive(Default)]
pub struct VerifyStore {
    tokens: Mutex<Vec<TokenRecord>>,
    by_sha: Mutex<HashMap<String, usize>>,
    queues: Mutex<HashMap<String, StationQueue>>,
    clones: Mutex<Vec<CloneReport>>,
}

/// The handle threaded through the router.
pub type SharedVerifyStore = Arc<VerifyStore>;

/// The domain separator for a token leaf.
const TOKEN_LEAF_DOMAIN: &[u8] = b"eel.mobile.verify.token.v1";

impl VerifyStore {
    /// Construct an empty store.
    pub fn new() -> SharedVerifyStore {
        Arc::new(Self::default())
    }

    /// The leaf hash for a token.
    fn leaf_hash(token_sha256: &str, constituency_code: Option<&str>) -> String {
        let mut h = Sha256::new();
        h.update(TOKEN_LEAF_DOMAIN);
        h.update([0x1e]);
        h.update(token_sha256.as_bytes());
        h.update([0x1e]);
        h.update(constituency_code.unwrap_or("").as_bytes());
        format!("sha256:{}", hex::encode(h.finalize()))
    }

    /// Record a token, returning the record.
    ///
    /// A token already recorded is not recorded twice: the second call returns the
    /// original, so a replayed submission does not inflate the leaf set.
    pub fn record_token(
        &self,
        token_sha256: &str,
        constituency_code: Option<String>,
        source: &str,
    ) -> TokenRecord {
        let mut guard = self.tokens.lock().expect("lock poisoned");
        if let Some(index) = self.by_sha.lock().expect("lock poisoned").get(token_sha256) {
            return guard[*index].clone();
        }
        let leaf_index = guard.len();
        let record = TokenRecord {
            token_sha256: token_sha256.to_string(),
            leaf_hash: Self::leaf_hash(token_sha256, constituency_code.as_deref()),
            constituency_code,
            source: source.to_string(),
            leaf_index,
            recorded_at: chrono::Utc::now().to_rfc3339(),
        };
        guard.push(record.clone());
        drop(guard);
        self.by_sha
            .lock()
            .expect("lock poisoned")
            .insert(token_sha256.to_string(), leaf_index);
        record
    }

    /// A token by its SHA-256.
    pub fn token(&self, token_sha256: &str) -> Option<TokenRecord> {
        let index = *self
            .by_sha
            .lock()
            .expect("lock poisoned")
            .get(token_sha256)?;
        self.tokens
            .lock()
            .expect("lock poisoned")
            .get(index)
            .cloned()
    }

    /// Every recorded token, in arrival order.
    pub fn tokens(&self) -> Vec<TokenRecord> {
        self.tokens.lock().expect("lock poisoned").clone()
    }

    /// Tokens issued for a constituency, in arrival order.
    pub fn tokens_for_constituency(&self, code: &str) -> Vec<TokenRecord> {
        self.tokens()
            .into_iter()
            .filter(|t| t.constituency_code.as_deref() == Some(code))
            .collect()
    }

    /// The Merkle tree over the recorded tokens, or `None` when there are none.
    ///
    /// An empty tree has no root worth publishing; a root over nothing is the
    /// zero-root problem again in a different guise.
    pub fn tree(&self) -> Option<MerkleTree> {
        let tokens = self.tokens();
        if tokens.is_empty() {
            return None;
        }
        let entries: Vec<&[u8]> = tokens.iter().map(|t| t.leaf_hash.as_bytes()).collect();
        Some(MerkleTree::build(&entries))
    }

    /// The inclusion proof for a recorded token, verified against the root.
    ///
    /// Returns `None` when the token is not recorded or the proof does not verify.
    /// A proof that would not verify is never returned.
    pub fn proof_for(&self, token_sha256: &str) -> Option<(Vec<String>, String, usize, usize)> {
        let record = self.token(token_sha256)?;
        let tree = self.tree()?;
        let proof = tree.inclusion_proof(record.leaf_index).ok()?;
        tree.verify_inclusion(record.leaf_index, record.leaf_hash.as_bytes(), &proof)
            .ok()?;
        Some((
            proof.iter().map(hex::encode).collect(),
            hex::encode(tree.root()),
            record.leaf_index,
            tree.size(),
        ))
    }

    /// Check a station's queue.
    ///
    /// Returns the queue's state and whether this check-in was a re-entry, which is
    /// what a duplicated device presents as.
    pub fn check_in(&self, station: &str, agent_hash: &str) -> (StationQueue, bool) {
        let mut guard = self.queues.lock().expect("lock poisoned");
        let queue = guard.entry(station.to_string()).or_default();
        let duplicate = queue.checked_in.iter().any(|a| a == agent_hash);
        if !duplicate {
            queue.checked_in.push(agent_hash.to_string());
        }
        queue.history.push(agent_hash.to_string());
        (queue.clone(), duplicate)
    }

    /// Check an agent out of a station's queue.
    pub fn check_out(&self, station: &str, agent_hash: &str) -> Option<StationQueue> {
        let mut guard = self.queues.lock().expect("lock poisoned");
        let queue = guard.get_mut(station)?;
        queue.checked_in.retain(|a| a != agent_hash);
        Some(queue.clone())
    }

    /// A station's queue, if it has one.
    pub fn queue(&self, station: &str) -> Option<StationQueue> {
        self.queues
            .lock()
            .expect("lock poisoned")
            .get(station)
            .cloned()
    }

    /// Anomaly flags derived from the queue state.
    ///
    /// These are computed, not stored-and-forgotten: a re-entry within a station
    /// is a duplicated-device signal, and it is raised from the state that shows
    /// the re-entry rather than returned as an empty list for every query.
    pub fn anomalies(&self) -> Vec<AnomalyFlag> {
        let mut out = Vec::new();
        for (station, queue) in self.queues.lock().expect("lock poisoned").iter() {
            let mut seen = std::collections::HashSet::new();
            for agent in &queue.history {
                if !seen.insert(agent.clone()) {
                    out.push(AnomalyFlag {
                        anomaly_id: {
                            // Derived from the station and agent, so the same
                            // duplicate raises the same flag rather than a new
                            // one on every read.
                            let digest = Sha256::digest(
                                format!("eel.mobile.verify.anomaly.v1|{station}|{agent}")
                                    .as_bytes(),
                            );
                            format!("anom-{}", hex::encode(&digest[..8]))
                        },
                        kind: "duplicate-check-in".to_string(),
                        station: station.clone(),
                        detail: format!(
                            "agent {} checked in more than once at this station",
                            &agent[..agent.len().min(12)]
                        ),
                        raised_at: chrono::Utc::now().to_rfc3339(),
                    });
                }
            }
        }
        out
    }

    /// File a clone report, returning it with its digest.
    pub fn file_clone(
        &self,
        package_name: &str,
        build_id: &str,
        findings: Vec<String>,
    ) -> CloneReport {
        let mut h = Sha256::new();
        h.update(b"eel.mobile.verify.clone-report.v1");
        h.update([0x1e]);
        h.update(package_name.as_bytes());
        h.update([0x1e]);
        h.update(build_id.as_bytes());
        for f in &findings {
            h.update([0x1e]);
            h.update(f.as_bytes());
        }
        let report = CloneReport {
            report_id: uuid::Uuid::now_v7().to_string(),
            package_name: package_name.to_string(),
            build_id: build_id.to_string(),
            findings,
            report_hash: format!("sha256:{}", hex::encode(h.finalize())),
            filed_at: chrono::Utc::now().to_rfc3339(),
        };
        self.clones
            .lock()
            .expect("lock poisoned")
            .push(report.clone());
        report
    }

    /// Every filed clone report.
    pub fn clones(&self) -> Vec<CloneReport> {
        self.clones.lock().expect("lock poisoned").clone()
    }
}

/// A short id from a UUID.
///
/// The slice is taken from the **end** of the hex, not the start. A UUIDv7's
/// leading hex is its millisecond timestamp, so two ids minted in the same
/// millisecond share a prefix and slicing from the front produces collisions --
/// which is exactly what `two_jobs_have_different_subjects` caught when it found
/// both jobs reporting the second subject.
fn short_id() -> String {
    let hex = uuid::Uuid::now_v7().simple().to_string();
    hex[hex.len() - 12..].to_string()
}
