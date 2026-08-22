//! QuorumCertificate: threshold signature aggregate from 2f+1 replicas.
//!
//! A QC certifies that QUORUM (= 3) replicas have signed the same
//! (msg_type, view_number, block_hash) tuple.
//!
//! Reference: Yin, Malkhi, Reiter, Gueta, Abraham.
//!   "HotStuff: BFT Consensus with Linearity and Responsiveness."
//!   PODC 2019. ACM, pp. 347-356.

use serde::{Deserialize, Serialize};

use crate::message::MessageType;
use crate::replica::ReplicaId;
use crate::{BftError, QUORUM};

/// A partial vote from one replica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub replica_id: ReplicaId,
    pub signature: Vec<u8>,
}

/// Threshold certificate: QUORUM votes over (msg_type, view_number, block_hash).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumCertificate {
    pub msg_type: MessageType,
    pub view_number: u64,
    pub block_hash: [u8; 32],
    pub votes: Vec<Vote>,
}

impl QuorumCertificate {
    /// Build a QC from collected votes. Returns error if fewer than QUORUM votes.
    pub fn from_votes(
        msg_type: MessageType,
        view_number: u64,
        block_hash: [u8; 32],
        votes: Vec<Vote>,
    ) -> Result<Self, BftError> {
        if votes.len() < QUORUM {
            return Err(BftError::InsufficientSignatures {
                got: votes.len(),
                required: QUORUM,
            });
        }
        Ok(Self {
            msg_type,
            view_number,
            block_hash,
            votes,
        })
    }

    /// Verify all votes in the QC against the provided public keys.
    ///
    /// Each vote signature must cover the canonical signing bytes
    /// (msg_type_index || view_number || block_hash).
    ///
    /// Returns Ok(()) when all QUORUM votes verify correctly.
    pub fn verify(
        &self,
        public_keys: &[(ReplicaId, ed25519_dalek::VerifyingKey)],
    ) -> Result<(), BftError> {
        use ed25519_dalek::{Signature, Verifier};

        if self.votes.len() < QUORUM {
            return Err(BftError::InsufficientSignatures {
                got: self.votes.len(),
                required: QUORUM,
            });
        }

        let signing_bytes = crate::message::HotStuffMessage::signing_bytes(
            &self.msg_type,
            self.view_number,
            &self.block_hash,
        );

        let mut verified = 0usize;
        for vote in &self.votes {
            let pk = public_keys
                .iter()
                .find(|(id, _)| *id == vote.replica_id)
                .map(|(_, pk)| pk)
                .ok_or_else(|| {
                    BftError::InvalidQc(format!("unknown replica {}", vote.replica_id))
                })?;

            let sig = Signature::from_slice(&vote.signature)
                .map_err(|e| BftError::Signature(e.to_string()))?;

            pk.verify(&signing_bytes, &sig)
                .map_err(|e| BftError::Signature(e.to_string()))?;

            verified += 1;
        }

        if verified < QUORUM {
            return Err(BftError::InsufficientSignatures {
                got: verified,
                required: QUORUM,
            });
        }

        Ok(())
    }

    /// High QC comparison: higher view wins.
    pub fn is_higher_than(&self, other: &Self) -> bool {
        self.view_number > other.view_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_vote(id: ReplicaId) -> Vote {
        Vote {
            replica_id: id,
            signature: vec![0u8; 64],
        }
    }

    #[test]
    fn qc_requires_quorum_votes() {
        let err = QuorumCertificate::from_votes(
            MessageType::Prepare,
            1,
            [0u8; 32],
            vec![dummy_vote(0), dummy_vote(1)], // only 2, need 3
        )
        .unwrap_err();
        assert!(matches!(
            err,
            BftError::InsufficientSignatures {
                got: 2,
                required: 3
            }
        ));
    }

    #[test]
    fn qc_accepts_quorum_votes() {
        let qc = QuorumCertificate::from_votes(
            MessageType::Prepare,
            1,
            [0u8; 32],
            vec![dummy_vote(0), dummy_vote(1), dummy_vote(2)],
        )
        .unwrap();
        assert_eq!(qc.votes.len(), 3);
    }

    #[test]
    fn higher_view_qc_wins() {
        let qc1 = QuorumCertificate {
            msg_type: MessageType::Prepare,
            view_number: 5,
            block_hash: [0u8; 32],
            votes: vec![],
        };
        let qc2 = QuorumCertificate {
            msg_type: MessageType::Prepare,
            view_number: 3,
            block_hash: [0u8; 32],
            votes: vec![],
        };
        assert!(qc1.is_higher_than(&qc2));
        assert!(!qc2.is_higher_than(&qc1));
    }
}
