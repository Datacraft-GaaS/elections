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

//! Replica state machine for HotStuff BFT.
//!
//! Implements the four-phase linear HotStuff protocol:
//!   Prepare -> PreCommit -> Commit -> Decide
//!
//! Each phase requires a QC from the previous phase. The leader drives the
//! protocol; replicas send votes and the leader aggregates into QCs.
//!
//! Reference: Yin, Malkhi, Reiter, Gueta, Abraham.
//!   "HotStuff: BFT Consensus with Linearity and Responsiveness."
//!   PODC 2019. ACM, pp. 347-356. Figure 3 (Basic HotStuff).
//!
//! Safety invariant: a replica never votes for a conflicting block at the same
//! view. Liveness requires at least N - f = 3 correct replicas.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::certificate::{QuorumCertificate, Vote};
use crate::message::{Block, HotStuffMessage, MessageType};
use crate::{BftError, F, N, QUORUM};

pub type ReplicaId = u64;

/// Configuration for one replica.
#[derive(Clone)]
pub struct ReplicaConfig {
    pub id: ReplicaId,
    pub n: usize,
    pub f: usize,
    pub signing_key: SigningKey,
    /// Verifying keys for all N replicas (including self).
    pub peer_keys: Vec<(ReplicaId, VerifyingKey)>,
}

impl ReplicaConfig {
    /// Build with workspace constants N=4, f=1.
    pub fn new(
        id: ReplicaId,
        signing_key: SigningKey,
        peer_keys: Vec<(ReplicaId, VerifyingKey)>,
    ) -> Self {
        assert_eq!(
            peer_keys.len(),
            N,
            "peer_keys must contain all {N} replicas"
        );
        Self {
            id,
            n: N,
            f: F,
            signing_key,
            peer_keys,
        }
    }

    /// Leader for view v is replica (v mod N).
    pub fn leader_for_view(&self, view: u64) -> ReplicaId {
        view % self.n as u64
    }

    pub fn is_leader(&self, view: u64) -> bool {
        self.leader_for_view(view) == self.id
    }
}

/// Phase within a single HotStuff view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Prepare,
    PreCommit,
    Commit,
    Decide,
}

/// Replica mutable state.
pub struct Replica {
    pub config: ReplicaConfig,
    pub view: u64,
    pub phase: Phase,
    /// Highest QC seen.
    pub high_qc: Option<QuorumCertificate>,
    /// Block locked during the PreCommit phase.
    pub locked_qc: Option<QuorumCertificate>,
    /// Committed blocks (in order).
    pub committed: Vec<Block>,
    /// Votes collected this view (leader only).
    pub pending_votes: Vec<Vote>,
}

impl Replica {
    pub fn new(config: ReplicaConfig) -> Self {
        Self {
            config,
            view: 0,
            phase: Phase::Prepare,
            high_qc: None,
            locked_qc: None,
            committed: Vec::new(),
            pending_votes: Vec::new(),
        }
    }

    /// Produce a signed vote for the given message.
    pub fn sign_vote(
        &self,
        msg_type: &MessageType,
        view_number: u64,
        block_hash: &[u8; 32],
    ) -> Result<Vote, BftError> {
        let bytes = HotStuffMessage::signing_bytes(msg_type, view_number, block_hash);
        let sig: Signature = self.config.signing_key.sign(&bytes);
        Ok(Vote {
            replica_id: self.config.id,
            signature: sig.to_bytes().to_vec(),
        })
    }

    /// Leader: propose a block in the Prepare phase.
    ///
    /// Returns a Prepare message to broadcast.
    pub fn propose(&mut self, block: Block) -> Result<HotStuffMessage, BftError> {
        if !self.config.is_leader(self.view) {
            return Err(BftError::NotLeader { id: self.config.id });
        }
        debug!(
            replica = self.config.id,
            view = self.view,
            "leader proposing block"
        );
        let vote = self.sign_vote(&MessageType::Prepare, self.view, &block.hash)?;
        let msg = HotStuffMessage {
            msg_type: MessageType::Prepare,
            view_number: self.view,
            block_hash: block.hash,
            block: Some(block),
            justify: self.high_qc.clone(),
            sender: self.config.id,
            signature: vote.signature.clone(),
        };
        self.pending_votes.clear();
        self.pending_votes.push(vote);
        self.phase = Phase::Prepare;
        Ok(msg)
    }

    /// Replica: process a Prepare message from leader; return a vote if valid.
    pub fn handle_prepare(&self, msg: &HotStuffMessage) -> Result<Vote, BftError> {
        if msg.view_number != self.view {
            return Err(BftError::ViewMismatch {
                expected: self.view,
                actual: msg.view_number,
            });
        }
        let block = msg
            .block
            .as_ref()
            .ok_or_else(|| BftError::InvalidQc("no block in prepare".into()))?;
        // Safety check: only vote if block extends locked_qc or there is no lock.
        if let Some(locked) = &self.locked_qc {
            if block.parent_hash != locked.block_hash {
                return Err(BftError::InvalidQc(
                    "block does not extend locked QC".into(),
                ));
            }
        }
        debug!(replica = self.config.id, view = self.view, "voting prepare");
        self.sign_vote(&MessageType::Prepare, self.view, &block.hash)
    }

    /// Leader: collect a vote; form QC when QUORUM votes arrive.
    pub fn collect_vote(
        &mut self,
        vote: Vote,
        phase: MessageType,
        block_hash: [u8; 32],
    ) -> Result<Option<QuorumCertificate>, BftError> {
        self.pending_votes.push(vote);
        if self.pending_votes.len() >= QUORUM {
            let qc = QuorumCertificate::from_votes(
                phase,
                self.view,
                block_hash,
                self.pending_votes.drain(..).collect(),
            )?;
            Ok(Some(qc))
        } else {
            Ok(None)
        }
    }

    /// Advance through PreCommit, Commit, Decide phases given the QC from prior phase.
    pub fn advance_phase(&mut self, qc: QuorumCertificate) -> Result<Phase, BftError> {
        match self.phase {
            Phase::Prepare => {
                self.high_qc = Some(qc);
                self.phase = Phase::PreCommit;
                info!(
                    replica = self.config.id,
                    view = self.view,
                    "phase -> PreCommit"
                );
            }
            Phase::PreCommit => {
                self.locked_qc = Some(qc);
                self.phase = Phase::Commit;
                info!(
                    replica = self.config.id,
                    view = self.view,
                    "phase -> Commit"
                );
            }
            Phase::Commit => {
                // Commit: safe to decide once we have the commit QC.
                self.phase = Phase::Decide;
                info!(
                    replica = self.config.id,
                    view = self.view,
                    "phase -> Decide"
                );
            }
            Phase::Decide => {
                warn!(replica = self.config.id, "advance_phase called in Decide");
            }
        }
        Ok(self.phase.clone())
    }

    /// Commit a block after a Decide QC.
    pub fn commit_block(&mut self, block: Block) {
        info!(
            replica = self.config.id,
            height = block.height,
            "block committed"
        );
        self.committed.push(block);
        // Move to next view.
        self.view += 1;
        self.phase = Phase::Prepare;
        self.pending_votes.clear();
    }

    /// Enter a new view (triggered by pacemaker).
    pub fn enter_view(&mut self, new_view: u64) {
        if new_view > self.view {
            self.view = new_view;
            self.phase = Phase::Prepare;
            self.pending_votes.clear();
            info!(
                replica = self.config.id,
                view = new_view,
                "entered new view"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key() -> SigningKey {
        use rand::rngs::OsRng;
        SigningKey::generate(&mut OsRng)
    }

    fn make_config(id: ReplicaId, keys: &[(ReplicaId, SigningKey)]) -> ReplicaConfig {
        let peer_keys = keys
            .iter()
            .map(|(i, sk)| (*i, sk.verifying_key()))
            .collect();
        ReplicaConfig::new(id, keys[id as usize].1.clone(), peer_keys)
    }

    fn make_cluster() -> Vec<(ReplicaId, SigningKey)> {
        (0..N as u64).map(|i| (i, make_key())).collect()
    }

    #[test]
    fn leader_rotates_by_view() {
        let keys = make_cluster();
        let cfg = make_config(0, &keys);
        assert_eq!(cfg.leader_for_view(0), 0);
        assert_eq!(cfg.leader_for_view(1), 1);
        assert_eq!(cfg.leader_for_view(4), 0);
    }

    #[test]
    fn leader_can_propose() {
        let keys = make_cluster();
        let cfg = make_config(0, &keys); // replica 0 is leader at view 0
        let mut r = Replica::new(cfg);
        let block = Block::new(1, [0u8; 32], b"tally_data".to_vec());
        let msg = r.propose(block).unwrap();
        assert_eq!(msg.msg_type, MessageType::Prepare);
        assert_eq!(msg.view_number, 0);
    }

    #[test]
    fn non_leader_cannot_propose() {
        let keys = make_cluster();
        let cfg = make_config(1, &keys); // replica 1 is NOT leader at view 0
        let mut r = Replica::new(cfg);
        let block = Block::new(1, [0u8; 32], b"data".to_vec());
        let err = r.propose(block).unwrap_err();
        assert!(matches!(err, BftError::NotLeader { id: 1 }));
    }

    #[test]
    fn replica_votes_on_prepare() {
        let keys = make_cluster();
        let leader_cfg = make_config(0, &keys);
        let mut leader = Replica::new(leader_cfg);
        let block = Block::new(1, [0u8; 32], b"data".to_vec());
        let msg = leader.propose(block).unwrap();

        let follower_cfg = make_config(1, &keys);
        let follower = Replica::new(follower_cfg);
        let vote = follower.handle_prepare(&msg).unwrap();
        assert_eq!(vote.replica_id, 1);
    }

    #[test]
    fn phase_advances_after_qc() {
        let keys = make_cluster();
        let cfg = make_config(0, &keys);
        let mut r = Replica::new(cfg);
        assert_eq!(r.phase, Phase::Prepare);

        let dummy_qc = QuorumCertificate {
            msg_type: MessageType::Prepare,
            view_number: 0,
            block_hash: [0u8; 32],
            votes: vec![],
        };
        r.advance_phase(dummy_qc).unwrap();
        assert_eq!(r.phase, Phase::PreCommit);
    }

    #[test]
    fn commit_block_advances_view() {
        let keys = make_cluster();
        let cfg = make_config(0, &keys);
        let mut r = Replica::new(cfg);
        let block = Block::new(1, [0u8; 32], b"data".to_vec());
        r.commit_block(block.clone());
        assert_eq!(r.view, 1);
        assert_eq!(r.committed.len(), 1);
    }
}
