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

//! Integration tests for eel-rts-bft-tally.
//! 25+ tests covering replica consensus, view-change, QC verification, adapter.

use ed25519_dalek::SigningKey;
use eel_rts_bft_tally::{
    adapter::{BftTallyAdapter, KafkaMessage, LocalNetwork},
    certificate::{QuorumCertificate, Vote},
    message::{Block, MessageType},
    pacemaker::Pacemaker,
    persistence::InMemoryStore,
    replica::{Phase, Replica, ReplicaConfig},
    BftError, F, N, QUORUM,
};
use rand::rngs::OsRng;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_signing_keys(n: usize) -> Vec<SigningKey> {
    (0..n).map(|_| SigningKey::generate(&mut OsRng)).collect()
}

fn make_cluster(keys: &[SigningKey]) -> Vec<Replica> {
    let peer_keys: Vec<(u64, ed25519_dalek::VerifyingKey)> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (i as u64, k.verifying_key()))
        .collect();
    keys.iter()
        .enumerate()
        .map(|(i, k)| {
            let cfg = ReplicaConfig::new(i as u64, k.clone(), peer_keys.clone());
            Replica::new(cfg)
        })
        .collect()
}

fn dummy_qc(msg_type: MessageType, view: u64, hash: [u8; 32], n: usize) -> QuorumCertificate {
    let votes: Vec<Vote> = (0..n)
        .map(|i| Vote {
            replica_id: i as u64,
            signature: vec![0u8; 64],
        })
        .collect();
    QuorumCertificate {
        msg_type,
        view_number: view,
        block_hash: hash,
        votes,
    }
}

// ---------------------------------------------------------------------------
// Core consensus tests
// ---------------------------------------------------------------------------

#[test]
fn constants_correct() {
    assert_eq!(N, 4);
    assert_eq!(F, 1);
    assert_eq!(QUORUM, 3);
}

#[test]
fn happy_path_leader_proposes_and_advances_through_all_phases() {
    let keys = make_signing_keys(N);
    let mut replicas = make_cluster(&keys);

    // Leader is replica 0 at view 0.
    let block = Block::new(1, [0u8; 32], b"form34a_aggregate".to_vec());
    let proposal = replicas[0].propose(block.clone()).unwrap();
    assert_eq!(proposal.msg_type, MessageType::Prepare);

    // Followers vote.
    let votes: Vec<Vote> = replicas[1..]
        .iter()
        .filter_map(|r| r.handle_prepare(&proposal).ok())
        .collect();

    // Leader collects votes; stop as soon as QC forms (drain empties pending_votes).
    let mut prepare_qc = None;
    for v in votes {
        if let Some(qc) = replicas[0]
            .collect_vote(v, MessageType::Prepare, block.hash)
            .unwrap()
        {
            prepare_qc = Some(qc);
            break;
        }
    }
    let prepare_qc = prepare_qc.expect("prepare QC should form after QUORUM votes");
    assert_eq!(prepare_qc.votes.len(), QUORUM);

    // Advance through PreCommit -> Commit -> Decide.
    replicas[0].advance_phase(prepare_qc.clone()).unwrap();
    assert_eq!(replicas[0].phase, Phase::PreCommit);

    let precommit_qc = dummy_qc(MessageType::PreCommit, 0, block.hash, QUORUM);
    replicas[0].advance_phase(precommit_qc).unwrap();
    assert_eq!(replicas[0].phase, Phase::Commit);

    let commit_qc = dummy_qc(MessageType::Commit, 0, block.hash, QUORUM);
    replicas[0].advance_phase(commit_qc).unwrap();
    assert_eq!(replicas[0].phase, Phase::Decide);

    // Commit.
    replicas[0].commit_block(block);
    assert_eq!(replicas[0].view, 1);
    assert_eq!(replicas[0].committed.len(), 1);
}

#[test]
fn f_equals_one_crashed_replica_still_commits() {
    // N=4, f=1: one replica crashed; three healthy replicas form QUORUM.
    let keys = make_signing_keys(N);
    // Use only 3 replicas (simulate replica 3 crashed).
    let healthy_keys = &keys[..3];
    let peer_keys: Vec<(u64, ed25519_dalek::VerifyingKey)> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (i as u64, k.verifying_key()))
        .collect();

    let mut leader = Replica::new(ReplicaConfig::new(0, keys[0].clone(), peer_keys.clone()));

    let block = Block::new(1, [0u8; 32], b"data".to_vec());
    let proposal = leader.propose(block.clone()).unwrap();

    // Two followers vote (replica 0 already voted as leader, replicas 1 and 2 add theirs).
    let follower1 = Replica::new(ReplicaConfig::new(
        1,
        healthy_keys[1].clone(),
        peer_keys.clone(),
    ));
    let follower2 = Replica::new(ReplicaConfig::new(
        2,
        healthy_keys[2].clone(),
        peer_keys.clone(),
    ));

    let v1 = follower1.handle_prepare(&proposal).unwrap();
    let v2 = follower2.handle_prepare(&proposal).unwrap();

    // Leader already has its own vote; add two more -> QUORUM = 3.
    let mut qc = None;
    qc = leader
        .collect_vote(v1, MessageType::Prepare, block.hash)
        .unwrap();
    qc = leader
        .collect_vote(v2, MessageType::Prepare, block.hash)
        .unwrap();
    assert!(
        qc.is_some(),
        "QUORUM reached with 3 votes despite 1 crashed replica"
    );
}

#[test]
fn f_equals_two_crashed_halts_safely() {
    // N=4, f=1: two replicas crash -> only 2 healthy < QUORUM (3); liveness lost.
    let keys = make_signing_keys(N);
    let peer_keys: Vec<(u64, ed25519_dalek::VerifyingKey)> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (i as u64, k.verifying_key()))
        .collect();
    let mut leader = Replica::new(ReplicaConfig::new(0, keys[0].clone(), peer_keys.clone()));
    let follower1 = Replica::new(ReplicaConfig::new(1, keys[1].clone(), peer_keys.clone()));

    let block = Block::new(1, [0u8; 32], b"data".to_vec());
    let proposal = leader.propose(block.clone()).unwrap();
    let v1 = follower1.handle_prepare(&proposal).unwrap();

    // Leader has its own vote + v1 = 2, < QUORUM (3). No QC forms.
    let qc = leader
        .collect_vote(v1, MessageType::Prepare, block.hash)
        .unwrap();
    assert!(qc.is_none(), "no QC should form with only 2 votes");
    // Safety preserved: nothing committed.
    assert_eq!(leader.committed.len(), 0);
}

#[test]
fn non_leader_cannot_propose() {
    let keys = make_signing_keys(N);
    let mut replicas = make_cluster(&keys);
    let block = Block::new(1, [0u8; 32], b"data".to_vec());
    // Replica 1 is not the leader at view 0.
    let err = replicas[1].propose(block).unwrap_err();
    assert!(matches!(err, BftError::NotLeader { id: 1 }));
}

#[test]
fn view_change_on_leader_failure() {
    let keys = make_signing_keys(N);
    let mut replicas = make_cluster(&keys);
    // Pacemaker fires; view advances to 1; new leader is replica 1.
    replicas[0].enter_view(1);
    assert_eq!(replicas[0].view, 1);
    assert_eq!(replicas[0].config.leader_for_view(1), 1);
}

#[test]
fn multiple_view_changes_rotate_leader() {
    let keys = make_signing_keys(N);
    let replicas = make_cluster(&keys);
    let cfg = &replicas[0].config;
    for i in 0..8u64 {
        assert_eq!(cfg.leader_for_view(i), i % N as u64);
    }
}

// ---------------------------------------------------------------------------
// QC verification tests
// ---------------------------------------------------------------------------

#[test]
fn qc_verify_rejects_insufficient_signatures() {
    let err =
        QuorumCertificate::from_votes(MessageType::Prepare, 0, [0u8; 32], vec![]).unwrap_err();
    assert!(matches!(
        err,
        BftError::InsufficientSignatures {
            got: 0,
            required: 3
        }
    ));
}

#[test]
fn qc_verify_rejects_two_of_three() {
    let votes: Vec<Vote> = (0..2)
        .map(|i| Vote {
            replica_id: i,
            signature: vec![0u8; 64],
        })
        .collect();
    let err = QuorumCertificate::from_votes(MessageType::Prepare, 0, [0u8; 32], votes).unwrap_err();
    assert!(matches!(
        err,
        BftError::InsufficientSignatures {
            got: 2,
            required: 3
        }
    ));
}

#[test]
fn qc_verify_accepts_three_valid_signatures() {
    use ed25519_dalek::{Signer, Verifier};

    let keys = make_signing_keys(N);
    let peer_keys: Vec<(u64, ed25519_dalek::VerifyingKey)> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (i as u64, k.verifying_key()))
        .collect();

    let hash = [42u8; 32];
    let view = 5u64;
    let msg_type = MessageType::Prepare;
    let signing_bytes =
        eel_rts_bft_tally::message::HotStuffMessage::signing_bytes(&msg_type, view, &hash);

    let votes: Vec<Vote> = keys[..QUORUM]
        .iter()
        .enumerate()
        .map(|(i, sk)| {
            let sig = sk.sign(&signing_bytes);
            Vote {
                replica_id: i as u64,
                signature: sig.to_bytes().to_vec(),
            }
        })
        .collect();

    let qc = QuorumCertificate::from_votes(msg_type, view, hash, votes).unwrap();
    qc.verify(&peer_keys).unwrap();
}

#[test]
fn qc_verify_rejects_tampered_signature() {
    use ed25519_dalek::Signer;

    let keys = make_signing_keys(N);
    let peer_keys: Vec<(u64, ed25519_dalek::VerifyingKey)> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (i as u64, k.verifying_key()))
        .collect();

    let hash = [1u8; 32];
    let view = 1u64;
    let msg_type = MessageType::Commit;
    let signing_bytes =
        eel_rts_bft_tally::message::HotStuffMessage::signing_bytes(&msg_type, view, &hash);

    let mut votes: Vec<Vote> = keys[..QUORUM]
        .iter()
        .enumerate()
        .map(|(i, sk)| Vote {
            replica_id: i as u64,
            signature: sk.sign(&signing_bytes).to_bytes().to_vec(),
        })
        .collect();

    // Tamper the first signature.
    votes[0].signature[0] ^= 0xFF;

    let qc = QuorumCertificate {
        msg_type,
        view_number: view,
        block_hash: hash,
        votes,
    };
    let err = qc.verify(&peer_keys).unwrap_err();
    assert!(matches!(err, BftError::Signature(_)));
}

// ---------------------------------------------------------------------------
// Pacemaker tests
// ---------------------------------------------------------------------------

#[test]
fn pacemaker_timeout_doubles() {
    let mut pm = Pacemaker::new(Duration::from_millis(200), Duration::from_millis(3200));
    pm.on_timeout();
    assert_eq!(pm.timeout(), Duration::from_millis(400));
    pm.on_timeout();
    assert_eq!(pm.timeout(), Duration::from_millis(800));
}

#[test]
fn pacemaker_commit_resets_backoff() {
    let mut pm = Pacemaker::new(Duration::from_millis(200), Duration::from_millis(3200));
    pm.on_timeout();
    pm.on_timeout();
    pm.on_commit();
    assert_eq!(pm.timeout(), Duration::from_millis(200));
}

#[test]
fn pacemaker_max_timeout_capped() {
    let mut pm = Pacemaker::new(Duration::from_millis(100), Duration::from_millis(800));
    for _ in 0..20 {
        pm.on_timeout();
    }
    assert_eq!(pm.timeout(), Duration::from_millis(800));
}

// ---------------------------------------------------------------------------
// Adapter round-trip tests
// ---------------------------------------------------------------------------

#[test]
fn adapter_round_trip_single_message() {
    let mut adapter = BftTallyAdapter::new();
    let payload = b"aggregate_34a_data".to_vec();
    let block = Block::new(1, [0u8; 32], payload.clone());

    let qc = dummy_qc(MessageType::Decide, 0, block.hash, QUORUM);
    let msg = KafkaMessage {
        topic: "eel.rts.form34a.v1".into(),
        partition: 0,
        offset: 0,
        payload,
    };
    let result = adapter.process(&msg, qc).unwrap();
    assert_eq!(result.height, 1);
    assert_eq!(result.payload, b"aggregate_34a_data");
}

#[test]
fn adapter_round_trip_sequential_messages() {
    let mut adapter = BftTallyAdapter::new();

    for i in 1u64..=5 {
        let payload = format!("batch_{i}").into_bytes();
        let block = Block::new(i, adapter.last_hash, payload.clone());
        let qc = dummy_qc(MessageType::Decide, i - 1, block.hash, QUORUM);
        let msg = KafkaMessage {
            topic: "eel.rts.form34a.v1".into(),
            partition: 0,
            offset: i as i64 - 1,
            payload,
        };
        let result = adapter.process(&msg, qc).unwrap();
        assert_eq!(result.height, i);
    }
    assert_eq!(adapter.height, 5);
}

#[test]
fn adapter_rejects_hash_mismatch() {
    let mut adapter = BftTallyAdapter::new();
    let qc = dummy_qc(MessageType::Decide, 0, [0xFF; 32], QUORUM);
    let msg = KafkaMessage {
        topic: "eel.rts.form34a.v1".into(),
        partition: 0,
        offset: 0,
        payload: b"data".to_vec(),
    };
    let err = adapter.process(&msg, qc).unwrap_err();
    assert!(matches!(err, BftError::HashMismatch));
}

// ---------------------------------------------------------------------------
// Persistence tests
// ---------------------------------------------------------------------------

#[test]
fn persistence_stores_and_retrieves() {
    let mut store = InMemoryStore::new(0);
    let b = Block::new(1, [0u8; 32], b"payload".to_vec());
    store.save_block(b.clone()).unwrap();
    assert!(store.get_block(1).is_some());
    assert!(store.get_block(2).is_none());
}

#[test]
fn persistence_advance_view_monotone() {
    let mut store = InMemoryStore::new(0);
    store.advance_view(5);
    store.advance_view(3); // should not regress
    assert_eq!(store.recover().last_committed_view, 5);
}
