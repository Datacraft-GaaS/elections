//! RFC 6962 test vectors and integration tests for verifier-rs.
//!
//! Builds Merkle trees with known leaves, generates proofs via eel-crypto-rs,
//! then verifies using the public verifier-rs API.  Also exercises STH
//! signature, witness cosign, and bundle hash.

use eel_crypto_rs::{
    blake3::hash_hex,
    ed25519::{generate_keypair, sign},
    merkle::MerkleTree,
    rfc8785::canonicalize,
};
use eel_verifier::{
    verify_bundle_hash, verify_consistency, verify_inclusion, verify_sth_signature,
    verify_witness_cosign, ConsistencyProof, InclusionProof, Sth, WitnessSig,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a signed Sth from a MerkleTree using a fresh generated keypair.
/// Returns (sth, signing_key_bytes).
fn make_signed_sth(tree: &MerkleTree, tree_size: usize) -> (Sth, [u8; 32]) {
    let root_hex = hex::encode(tree.root());
    let timestamp = 1_700_000_000u64;

    // The message is RFC 8785 canonical JSON of {sha256_root_hash, timestamp, tree_size}.
    let v = serde_json::json!({
        "sha256_root_hash": &root_hex,
        "timestamp": timestamp,
        "tree_size": tree_size as u64,
    });
    let msg = canonicalize(&v).expect("canonicalize");

    let kp = generate_keypair();
    let sig = sign(&kp, &msg);

    let sth = Sth {
        tree_size: tree_size as u64,
        timestamp,
        sha256_root_hash: root_hex,
        tree_head_signature: hex::encode(sig.to_bytes()),
        log_key: hex::encode(kp.verifying_key().to_bytes()),
    };
    (sth, kp.to_bytes())
}

fn make_inclusion_proof(tree: &MerkleTree, index: usize) -> InclusionProof {
    let hashes = tree
        .inclusion_proof(index)
        .expect("inclusion_proof")
        .iter()
        .map(hex::encode)
        .collect();
    InclusionProof {
        leaf_index: index as u64,
        tree_size: tree.size() as u64,
        hashes,
    }
}

fn make_consistency_proof_structs(
    old_tree: &MerkleTree,
    new_tree: &MerkleTree,
) -> (Sth, Sth, ConsistencyProof) {
    let (old_sth, _) = make_signed_sth(old_tree, old_tree.size());
    let (new_sth, _) = make_signed_sth(new_tree, new_tree.size());
    let hashes = new_tree
        .consistency_proof(old_tree.size())
        .expect("consistency_proof")
        .iter()
        .map(hex::encode)
        .collect();
    let proof = ConsistencyProof {
        old_size: old_tree.size() as u64,
        new_size: new_tree.size() as u64,
        hashes,
    };
    (old_sth, new_sth, proof)
}

// ---------------------------------------------------------------------------
// Inclusion proof tests
// ---------------------------------------------------------------------------

#[test]
fn inclusion_single_leaf() {
    let entries: Vec<&[u8]> = vec![b"alpha"];
    let tree = MerkleTree::build(&entries);
    let (sth, _) = make_signed_sth(&tree, 1);
    let proof = make_inclusion_proof(&tree, 0);
    verify_inclusion(&sth, &proof, b"alpha").expect("verify_inclusion single leaf");
}

#[test]
fn inclusion_two_leaves_each_index() {
    let entries: Vec<&[u8]> = vec![b"a", b"b"];
    let tree = MerkleTree::build(&entries);
    let (sth, _) = make_signed_sth(&tree, 2);

    let proof0 = make_inclusion_proof(&tree, 0);
    verify_inclusion(&sth, &proof0, b"a").expect("index 0");

    let proof1 = make_inclusion_proof(&tree, 1);
    verify_inclusion(&sth, &proof1, b"b").expect("index 1");
}

#[test]
fn inclusion_five_leaves() {
    let raw: Vec<&[u8]> = vec![b"alpha", b"beta", b"gamma", b"delta", b"epsilon"];
    let tree = MerkleTree::build(&raw);
    let (sth, _) = make_signed_sth(&tree, raw.len());
    for (i, leaf) in raw.iter().enumerate() {
        let proof = make_inclusion_proof(&tree, i);
        verify_inclusion(&sth, &proof, leaf)
            .unwrap_or_else(|e| panic!("leaf {i} inclusion failed: {e}"));
    }
}

#[test]
fn inclusion_wrong_data_fails() {
    let entries: Vec<&[u8]> = vec![b"real", b"data"];
    let tree = MerkleTree::build(&entries);
    let (sth, _) = make_signed_sth(&tree, 2);
    let proof = make_inclusion_proof(&tree, 0);
    assert!(
        verify_inclusion(&sth, &proof, b"tampered").is_err(),
        "tampered leaf must not verify"
    );
}

#[test]
fn inclusion_wrong_root_fails() {
    let entries: Vec<&[u8]> = vec![b"a", b"b"];
    let tree = MerkleTree::build(&entries);
    let (mut sth, _) = make_signed_sth(&tree, 2);
    // Corrupt the root hash.
    sth.sha256_root_hash = "00".repeat(32);
    let proof = make_inclusion_proof(&tree, 0);
    assert!(verify_inclusion(&sth, &proof, b"a").is_err());
}

// ---------------------------------------------------------------------------
// Consistency proof tests
// ---------------------------------------------------------------------------

#[test]
fn consistency_grow_2_to_4() {
    let old_leaves: Vec<&[u8]> = vec![b"a", b"b"];
    let new_leaves: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
    let old_tree = MerkleTree::build(&old_leaves);
    let new_tree = MerkleTree::build(&new_leaves);
    let (old_sth, new_sth, proof) = make_consistency_proof_structs(&old_tree, &new_tree);
    verify_consistency(&old_sth, &new_sth, &proof).expect("consistency 2->4");
}

#[test]
fn consistency_grow_1_to_7() {
    let leaves: Vec<Vec<u8>> = (0u8..7).map(|i| vec![i]).collect();
    let refs: Vec<&[u8]> = leaves.iter().map(Vec::as_slice).collect();
    let old_tree = MerkleTree::build(&refs[..1]);
    let new_tree = MerkleTree::build(&refs);
    let (old_sth, new_sth, proof) = make_consistency_proof_structs(&old_tree, &new_tree);
    verify_consistency(&old_sth, &new_sth, &proof).expect("consistency 1->7");
}

#[test]
fn consistency_same_size() {
    let leaves: Vec<&[u8]> = vec![b"x", b"y"];
    let tree = MerkleTree::build(&leaves);
    let (sth, _) = make_signed_sth(&tree, 2);
    let proof = ConsistencyProof {
        old_size: 2,
        new_size: 2,
        hashes: vec![],
    };
    verify_consistency(&sth, &sth, &proof).expect("same-size consistency");
}

#[test]
fn consistency_tampered_new_root_fails() {
    let old_leaves: Vec<&[u8]> = vec![b"a", b"b"];
    let new_leaves: Vec<&[u8]> = vec![b"a", b"b", b"c"];
    let old_tree = MerkleTree::build(&old_leaves);
    let new_tree = MerkleTree::build(&new_leaves);
    let (old_sth, mut new_sth, proof) = make_consistency_proof_structs(&old_tree, &new_tree);
    new_sth.sha256_root_hash = "ff".repeat(32);
    assert!(verify_consistency(&old_sth, &new_sth, &proof).is_err());
}

// ---------------------------------------------------------------------------
// STH signature tests
// ---------------------------------------------------------------------------

#[test]
fn sth_signature_verifies() {
    let entries: Vec<&[u8]> = vec![b"vote1", b"vote2", b"vote3"];
    let tree = MerkleTree::build(&entries);
    let (sth, _) = make_signed_sth(&tree, 3);
    verify_sth_signature(&sth, None).expect("STH signature must verify");
}

#[test]
fn sth_signature_wrong_key_fails() {
    let entries: Vec<&[u8]> = vec![b"data"];
    let tree = MerkleTree::build(&entries);
    let (mut sth, _) = make_signed_sth(&tree, 1);
    // Replace log_key with a different (random) key.
    let other_kp = generate_keypair();
    sth.log_key = hex::encode(other_kp.verifying_key().to_bytes());
    assert!(verify_sth_signature(&sth, None).is_err());
}

#[test]
fn sth_signature_key_override() {
    let entries: Vec<&[u8]> = vec![b"leaf"];
    let tree = MerkleTree::build(&entries);
    let (sth, _) = make_signed_sth(&tree, 1);
    // Override with the correct key (same as sth.log_key) should still verify.
    let key_clone = sth.log_key.clone();
    verify_sth_signature(&sth, Some(&key_clone)).expect("key_override with correct key");
}

// ---------------------------------------------------------------------------
// Witness cosign tests
// ---------------------------------------------------------------------------

#[test]
fn witness_cosign_empty_is_ok() {
    let entries: Vec<&[u8]> = vec![b"leaf"];
    let tree = MerkleTree::build(&entries);
    let (sth, _) = make_signed_sth(&tree, 1);
    verify_witness_cosign(&sth, &[]).expect("empty witness set");
}

#[test]
fn witness_cosign_single_valid() {
    let entries: Vec<&[u8]> = vec![b"leaf"];
    let tree = MerkleTree::build(&entries);
    let (sth, _) = make_signed_sth(&tree, 1);

    // Build the same signing message to generate a witness signature.
    let v = serde_json::json!({
        "sha256_root_hash": &sth.sha256_root_hash,
        "timestamp": sth.timestamp,
        "tree_size": sth.tree_size,
    });
    let msg = canonicalize(&v).expect("canonicalize");
    let witness_kp = generate_keypair();
    let witness_sig = sign(&witness_kp, &msg);

    let ws = WitnessSig {
        key: hex::encode(witness_kp.verifying_key().to_bytes()),
        signature: hex::encode(witness_sig.to_bytes()),
    };
    verify_witness_cosign(&sth, &[ws]).expect("single witness cosign");
}

#[test]
fn witness_cosign_invalid_sig_fails() {
    let entries: Vec<&[u8]> = vec![b"leaf"];
    let tree = MerkleTree::build(&entries);
    let (sth, _) = make_signed_sth(&tree, 1);

    let witness_kp = generate_keypair();
    // Signature over wrong message.
    let bad_sig = sign(&witness_kp, b"wrong message");

    let ws = WitnessSig {
        key: hex::encode(witness_kp.verifying_key().to_bytes()),
        signature: hex::encode(bad_sig.to_bytes()),
    };
    assert!(verify_witness_cosign(&sth, &[ws]).is_err());
}

// ---------------------------------------------------------------------------
// Bundle hash tests
// ---------------------------------------------------------------------------

#[test]
fn bundle_hash_correct() {
    let data = b"election bundle data";
    let expected = hash_hex(data);
    verify_bundle_hash(data, &expected).expect("bundle hash");
}

#[test]
fn bundle_hash_mismatch_fails() {
    let data = b"original data";
    let expected = hash_hex(b"different data");
    assert!(verify_bundle_hash(data, &expected).is_err());
}

#[test]
fn bundle_hash_empty_input() {
    let data = b"";
    let expected = hash_hex(b"");
    verify_bundle_hash(data, &expected).expect("empty bundle hash");
}
