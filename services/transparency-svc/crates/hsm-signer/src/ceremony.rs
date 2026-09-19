//! M-of-N ceremony state machine (ADR-0005).
//!
//! States: Requested -> AwaitingQuorum{approvals} -> Approved -> Executing -> Complete | Aborted
//!
//! Custodian approvals are Ed25519 signatures over (ceremony_id || operation_hash).
//! Any custodian may abort while in Requested or AwaitingQuorum.
//! Persistence: in-memory + advisory write to PG via outbox (full PG durability is follow-on).

use std::sync::Arc;

use chrono::Utc;
use dashmap::DashMap;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::broadcast;
use uuid::Uuid;

use eel_crypto_rs::KeyRole;

/// A registered custodian with their Ed25519 public key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Custodian {
    pub id: String,
    pub name: String,
    /// Raw 32-byte Ed25519 public key.
    pub pubkey: [u8; 32],
    pub registered_at: String,
}

/// One custodian's approval of a ceremony.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustodianApproval {
    pub custodian_id: String,
    pub signature: Vec<u8>,
    pub approved_at: String,
}

/// Current state of a ceremony.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CeremonyState {
    Requested,
    AwaitingQuorum { approvals: Vec<CustodianApproval> },
    Approved,
    Executing,
    Complete,
    Aborted { reason: String },
}

impl CeremonyState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Requested => "Requested",
            Self::AwaitingQuorum { .. } => "AwaitingQuorum",
            Self::Approved => "Approved",
            Self::Executing => "Executing",
            Self::Complete => "Complete",
            Self::Aborted { .. } => "Aborted",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Aborted { .. })
    }
}

/// An active ceremony record.
#[derive(Debug, Clone)]
pub struct Ceremony {
    pub id: String,
    pub role: KeyRole,
    pub purpose: String,
    pub quorum_m: u32,
    pub quorum_n: u32,
    /// SHA-256 hex over (ceremony_id || role_label || purpose).
    pub operation_hash: String,
    pub state: CeremonyState,
    pub created_at: String,
}

impl Ceremony {
    fn compute_hash(id: &str, role: KeyRole, purpose: &str) -> String {
        let mut h = Sha256::new();
        h.update(id.as_bytes());
        h.update(role.label().as_bytes());
        h.update(purpose.as_bytes());
        hex::encode(h.finalize())
    }

    pub fn new(role: KeyRole, purpose: String, quorum_m: u32, quorum_n: u32) -> Self {
        let id = Uuid::new_v4().to_string();
        let operation_hash = Self::compute_hash(&id, role, &purpose);
        Self {
            id,
            role,
            purpose,
            quorum_m,
            quorum_n,
            operation_hash,
            state: CeremonyState::Requested,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    /// Returns current approval count if in AwaitingQuorum, else 0.
    pub fn approval_count(&self) -> u32 {
        match &self.state {
            CeremonyState::AwaitingQuorum { approvals } => approvals.len() as u32,
            _ => 0,
        }
    }
}

/// Event broadcast to ceremony subscribers.
#[derive(Debug, Clone)]
pub struct CeremonyUpdate {
    pub ceremony_id: String,
    pub state_label: String,
    pub approval: Option<CustodianApproval>,
    pub abort_reason: Option<String>,
    pub approvals_count: u32,
    pub quorum_m: u32,
    pub operation_hash: String,
}

/// Central store for ceremonies and registered custodians.
pub struct CeremonyStore {
    ceremonies: Arc<DashMap<String, Ceremony>>,
    custodians: Arc<DashMap<String, Custodian>>,
    // broadcast channel per ceremony_id
    channels: Arc<DashMap<String, broadcast::Sender<CeremonyUpdate>>>,
}

impl CeremonyStore {
    pub fn new() -> Self {
        Self {
            ceremonies: Arc::new(DashMap::new()),
            custodians: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
        }
    }

    /// Register an Ed25519 custodian public key.
    pub fn register_custodian(&self, name: String, pubkey: [u8; 32]) -> Custodian {
        let id = Uuid::new_v4().to_string();
        let c = Custodian {
            id: id.clone(),
            name,
            pubkey,
            registered_at: Utc::now().to_rfc3339(),
        };
        self.custodians.insert(id, c.clone());
        c
    }

    pub fn list_custodians(&self) -> Vec<Custodian> {
        self.custodians.iter().map(|e| e.value().clone()).collect()
    }

    /// Open a new ceremony and return its id and a subscriber.
    pub fn open(
        &self,
        role: KeyRole,
        purpose: String,
        quorum_m: u32,
        quorum_n: u32,
    ) -> (Ceremony, broadcast::Receiver<CeremonyUpdate>) {
        let ceremony = Ceremony::new(role, purpose, quorum_m, quorum_n);
        let (tx, rx) = broadcast::channel(64);
        self.channels.insert(ceremony.id.clone(), tx);
        self.ceremonies
            .insert(ceremony.id.clone(), ceremony.clone());
        (ceremony, rx)
    }

    /// Subscribe to an existing ceremony.
    pub fn subscribe(&self, ceremony_id: &str) -> Option<broadcast::Receiver<CeremonyUpdate>> {
        self.channels.get(ceremony_id).map(|tx| tx.subscribe())
    }

    /// Record a custodian approval.
    ///
    /// Verifies Ed25519 signature over (ceremony_id || operation_hash).
    /// Returns error string on failure.
    pub fn approve(
        &self,
        ceremony_id: &str,
        custodian_id: &str,
        sig_bytes: &[u8],
    ) -> Result<CeremonyUpdate, String> {
        let custodian = self
            .custodians
            .get(custodian_id)
            .ok_or_else(|| format!("custodian {custodian_id} not registered"))?;

        let mut ceremony = self
            .ceremonies
            .get_mut(ceremony_id)
            .ok_or_else(|| format!("ceremony {ceremony_id} not found"))?;

        // Must be in Requested or AwaitingQuorum.
        match &ceremony.state {
            CeremonyState::Requested | CeremonyState::AwaitingQuorum { .. } => {}
            other => return Err(format!("ceremony in terminal state {:?}", other.label())),
        }

        // Verify Ed25519.
        let verifying_key = VerifyingKey::from_bytes(&custodian.pubkey)
            .map_err(|e| format!("invalid custodian key: {e}"))?;
        let signature = Signature::from_slice(sig_bytes)
            .map_err(|e| format!("invalid signature bytes: {e}"))?;
        let mut message = ceremony_id.as_bytes().to_vec();
        message.extend_from_slice(ceremony.operation_hash.as_bytes());
        use ed25519_dalek::Verifier;
        verifying_key
            .verify(&message, &signature)
            .map_err(|e| format!("signature verification failed: {e}"))?;

        let approval = CustodianApproval {
            custodian_id: custodian_id.to_string(),
            signature: sig_bytes.to_vec(),
            approved_at: Utc::now().to_rfc3339(),
        };

        // Transition state.
        let new_state = match std::mem::replace(&mut ceremony.state, CeremonyState::Requested) {
            CeremonyState::Requested => {
                let approvals = vec![approval.clone()];
                if approvals.len() as u32 >= ceremony.quorum_m {
                    CeremonyState::Approved
                } else {
                    CeremonyState::AwaitingQuorum { approvals }
                }
            }
            CeremonyState::AwaitingQuorum { mut approvals } => {
                approvals.push(approval.clone());
                if approvals.len() as u32 >= ceremony.quorum_m {
                    CeremonyState::Approved
                } else {
                    CeremonyState::AwaitingQuorum { approvals }
                }
            }
            other => other,
        };
        ceremony.state = new_state;

        let update = CeremonyUpdate {
            ceremony_id: ceremony_id.to_string(),
            state_label: ceremony.state.label().to_string(),
            approval: Some(approval),
            abort_reason: None,
            approvals_count: ceremony.approval_count(),
            quorum_m: ceremony.quorum_m,
            operation_hash: ceremony.operation_hash.clone(),
        };

        if let Some(tx) = self.channels.get(ceremony_id) {
            let _ = tx.send(update.clone());
        }
        Ok(update)
    }

    /// Abort a ceremony (only from Requested or AwaitingQuorum).
    pub fn abort(&self, ceremony_id: &str, reason: &str) -> Result<CeremonyUpdate, String> {
        let mut ceremony = self
            .ceremonies
            .get_mut(ceremony_id)
            .ok_or_else(|| format!("ceremony {ceremony_id} not found"))?;

        match &ceremony.state {
            CeremonyState::Requested | CeremonyState::AwaitingQuorum { .. } => {}
            other => return Err(format!("cannot abort ceremony in state {}", other.label())),
        }

        ceremony.state = CeremonyState::Aborted {
            reason: reason.to_string(),
        };

        let update = CeremonyUpdate {
            ceremony_id: ceremony_id.to_string(),
            state_label: ceremony.state.label().to_string(),
            approval: None,
            abort_reason: Some(reason.to_string()),
            approvals_count: 0,
            quorum_m: ceremony.quorum_m,
            operation_hash: ceremony.operation_hash.clone(),
        };

        if let Some(tx) = self.channels.get(ceremony_id) {
            let _ = tx.send(update.clone());
        }
        Ok(update)
    }

    /// Transition an Approved ceremony to Executing, then Complete.
    pub fn mark_executing(&self, ceremony_id: &str) -> Result<(), String> {
        let mut ceremony = self
            .ceremonies
            .get_mut(ceremony_id)
            .ok_or_else(|| format!("ceremony {ceremony_id} not found"))?;

        if ceremony.state != CeremonyState::Approved {
            return Err(format!("ceremony not in Approved state"));
        }
        ceremony.state = CeremonyState::Executing;
        Ok(())
    }

    pub fn mark_complete(&self, ceremony_id: &str) -> Result<CeremonyUpdate, String> {
        let mut ceremony = self
            .ceremonies
            .get_mut(ceremony_id)
            .ok_or_else(|| format!("ceremony {ceremony_id} not found"))?;

        ceremony.state = CeremonyState::Complete;

        let update = CeremonyUpdate {
            ceremony_id: ceremony_id.to_string(),
            state_label: "Complete".to_string(),
            approval: None,
            abort_reason: None,
            approvals_count: 0,
            quorum_m: ceremony.quorum_m,
            operation_hash: ceremony.operation_hash.clone(),
        };

        if let Some(tx) = self.channels.get(ceremony_id) {
            let _ = tx.send(update.clone());
        }
        Ok(update)
    }

    pub fn get(&self, ceremony_id: &str) -> Option<Ceremony> {
        self.ceremonies.get(ceremony_id).map(|e| e.clone())
    }

    /// Verify that a ceremony is in Approved state (used by RotateKey gate).
    pub fn assert_approved(&self, ceremony_id: &str) -> Result<Ceremony, String> {
        let c = self
            .get(ceremony_id)
            .ok_or_else(|| format!("ceremony {ceremony_id} not found"))?;
        if c.state != CeremonyState::Approved {
            return Err(format!(
                "ceremony {} is in state {}, expected Approved",
                ceremony_id,
                c.state.label()
            ));
        }
        Ok(c)
    }
}

impl Default for CeremonyStore {
    fn default() -> Self {
        Self::new()
    }
}
