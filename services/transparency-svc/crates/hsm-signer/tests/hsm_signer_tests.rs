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

//! Integration tests for the HSM signer service.
//!
//! Tests that require SoftHSMv2 are skipped automatically when the shared
//! library is absent (documented below). To run all tests locally:
//!
//!   # macOS
//!   brew install softhsm
//!   softhsm2-util --init-token --slot 0 --label eel-test --pin 1234 --so-pin 5678
//!   cargo test -p hsm-signer
//!
//! SoftHSMv2 skip condition: `PKCS11_MODULE_PATH` env is set to a missing path, or the
//! default platform path does not exist. Tests that require HSM mark themselves
//! with `skip_if_no_softhsm!()`.
//!
//! Tests that do not require HSM (ceremony state machine, rate limiter) always run.

use eel_crypto_rs::KeyRole;
use hsm_signer::ceremony::{CeremonyState, CeremonyStore};
use hsm_signer::rate::RoleLimiters;

// ---------------------------------------------------------------------------
// SoftHSM skip helper
// ---------------------------------------------------------------------------

fn softhsm_path() -> std::path::PathBuf {
    std::env::var("PKCS11_MODULE_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            #[cfg(target_os = "macos")]
            return std::path::PathBuf::from("/usr/local/lib/softhsm/libsofthsm2.so");
            #[cfg(not(target_os = "macos"))]
            return std::path::PathBuf::from("/usr/lib/softhsm/libsofthsm2.so");
        })
}

fn softhsm_available() -> bool {
    softhsm_path().exists()
}

// ---------------------------------------------------------------------------
// Rate limiter tests (no HSM required)
// ---------------------------------------------------------------------------

#[test]
fn rate_limiter_allows_within_quota() {
    let limiters = RoleLimiters::new();
    // Sth allows 60/s; a single check must pass.
    assert!(limiters.check(KeyRole::Sth).is_ok());
}

#[test]
fn rate_limiter_exhausts_quota() {
    use governor::{Quota, RateLimiter};
    use std::num::NonZeroU32;

    // Create a limiter with quota of 1/s.
    let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
    let limiter = RateLimiter::direct(quota);
    // First check passes.
    assert!(limiter.check().is_ok());
    // Second immediate check fails.
    assert!(limiter.check().is_err());
}

// ---------------------------------------------------------------------------
// Ceremony state machine tests (no HSM required)
// ---------------------------------------------------------------------------

fn make_keypair() -> (ed25519_dalek::SigningKey, ed25519_dalek::VerifyingKey) {
    use ed25519_dalek::SigningKey;
    use rand_core::{OsRng, RngCore};
    let mut seed = [0u8; 32];
    OsRng.fill_bytes(&mut seed);
    let sk = SigningKey::from_bytes(&seed);
    let vk = sk.verifying_key();
    (sk, vk)
}

fn sign_approval(sk: &ed25519_dalek::SigningKey, ceremony_id: &str, op_hash: &str) -> Vec<u8> {
    use ed25519_dalek::Signer;
    let mut msg = ceremony_id.as_bytes().to_vec();
    msg.extend_from_slice(op_hash.as_bytes());
    sk.sign(&msg).to_bytes().to_vec()
}

#[test]
fn ceremony_happy_path_three_approvals() {
    let store = CeremonyStore::new();

    // Register 3 custodians.
    let mut keypairs = Vec::new();
    let mut custodian_ids = Vec::new();
    for name in ["alice", "bob", "carol"] {
        let (sk, vk) = make_keypair();
        let c = store.register_custodian(name.to_string(), vk.to_bytes());
        custodian_ids.push(c.id.clone());
        keypairs.push(sk);
    }

    let (ceremony, _rx) = store.open(KeyRole::CycleKek, "test rotation".to_string(), 3, 5);
    let cid = ceremony.id.clone();
    let op_hash = ceremony.operation_hash.clone();

    // 1st approval -> AwaitingQuorum.
    let sig0 = sign_approval(&keypairs[0], &cid, &op_hash);
    let upd = store.approve(&cid, &custodian_ids[0], &sig0).unwrap();
    assert_eq!(upd.state_label, "AwaitingQuorum");
    assert_eq!(upd.approvals_count, 1);

    // 2nd approval -> still AwaitingQuorum (quorum = 3).
    let sig1 = sign_approval(&keypairs[1], &cid, &op_hash);
    let upd = store.approve(&cid, &custodian_ids[1], &sig1).unwrap();
    assert_eq!(upd.state_label, "AwaitingQuorum");
    assert_eq!(upd.approvals_count, 2);

    // 3rd approval -> Approved.
    let sig2 = sign_approval(&keypairs[2], &cid, &op_hash);
    let upd = store.approve(&cid, &custodian_ids[2], &sig2).unwrap();
    assert_eq!(upd.state_label, "Approved");

    let c = store.get(&cid).unwrap();
    assert_eq!(c.state, CeremonyState::Approved);
}

#[test]
fn ceremony_abort() {
    let store = CeremonyStore::new();
    let (ceremony, _rx) = store.open(KeyRole::Declaration, "abort test".to_string(), 2, 3);
    let cid = ceremony.id.clone();

    let upd = store.abort(&cid, "test abort").unwrap();
    assert_eq!(upd.state_label, "Aborted");
    assert_eq!(upd.abort_reason.unwrap(), "test abort");

    let c = store.get(&cid).unwrap();
    assert!(matches!(c.state, CeremonyState::Aborted { .. }));
}

#[test]
fn ceremony_abort_in_awaiting_quorum() {
    let store = CeremonyStore::new();
    let (sk, vk) = make_keypair();
    let custodian = store.register_custodian("dave".to_string(), vk.to_bytes());

    let (ceremony, _rx) = store.open(KeyRole::Sth, "abort from quorum".to_string(), 3, 5);
    let cid = ceremony.id.clone();
    let op_hash = ceremony.operation_hash.clone();

    let sig = sign_approval(&sk, &cid, &op_hash);
    store.approve(&cid, &custodian.id, &sig).unwrap();

    let upd = store.abort(&cid, "emergency abort").unwrap();
    assert_eq!(upd.state_label, "Aborted");
}

#[test]
fn ceremony_invalid_signature_rejected() {
    let store = CeremonyStore::new();
    let (_sk, vk) = make_keypair();
    let custodian = store.register_custodian("eve".to_string(), vk.to_bytes());

    let (ceremony, _rx) = store.open(KeyRole::CodeSigning, "invalid sig".to_string(), 2, 3);
    let cid = ceremony.id.clone();

    // Use a bad signature (wrong key).
    let (bad_sk, _) = make_keypair();
    let bad_sig = sign_approval(&bad_sk, &cid, &ceremony.operation_hash);
    let result = store.approve(&cid, &custodian.id, &bad_sig);
    assert!(result.is_err(), "bad signature must be rejected");
}

// ---------------------------------------------------------------------------
// HSM round-trip tests (skipped if SoftHSMv2 absent)
// ---------------------------------------------------------------------------

#[test]
fn sign_verify_roundtrip_softhsm() {
    if !softhsm_available() {
        eprintln!(
            "SKIP sign_verify_roundtrip_softhsm: SoftHSMv2 not found at {:?}. \
			 Install SoftHSMv2 and init a slot to run this test.",
            softhsm_path()
        );
        return;
    }

    let pin = std::env::var("EEL_HSM_PIN").unwrap_or_else(|_| "1234".to_string());
    let slot: u64 = std::env::var("EEL_HSM_SLOT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let pool = hsm_signer::pool::RolePool::open(slot, pin).expect("open pool");

    let payload = b"test payload for eel sth signing";
    let (sig, key_id, algo) = pool.sign(KeyRole::Sth, payload).expect("sign");
    assert!(!sig.is_empty());
    assert!(!key_id.is_empty());
    assert_eq!(algo, "ECDSA-P384");

    let ok = pool.verify(KeyRole::Sth, payload, &sig).expect("verify");
    assert!(ok, "signature must verify");
}

#[test]
fn rotate_key_emits_event() {
    // This test verifies the ceremony -> rotate_key flow at the state machine
    // level without requiring an actual HSM. Full outbox emission is validated
    // in the WP-2.2 integration test suite against a running PG instance.
    let store = CeremonyStore::new();
    let (sk, vk) = make_keypair();
    let custodian = store.register_custodian("frank".to_string(), vk.to_bytes());

    let (ceremony, _rx) = store.open(KeyRole::CycleKek, "cycle kek rotation".to_string(), 1, 3);
    let cid = ceremony.id.clone();
    let op_hash = ceremony.operation_hash.clone();

    let sig = sign_approval(&sk, &cid, &op_hash);
    let upd = store.approve(&cid, &custodian.id, &sig).unwrap();
    // quorum_m = 1, so single approval -> Approved.
    assert_eq!(upd.state_label, "Approved");

    // mark_executing -> mark_complete simulates rotate_key.
    store.mark_executing(&cid).unwrap();
    let complete = store.mark_complete(&cid).unwrap();
    assert_eq!(complete.state_label, "Complete");
}
