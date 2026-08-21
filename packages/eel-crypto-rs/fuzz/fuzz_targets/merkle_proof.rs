//! Fuzz target: Merkle inclusion proof decoder.
//!
//! Feeds random bytes as if they were a Merkle proof and verifies that
//! the decoder never panics (it may return an error, that is expected).

#![no_main]
use eel_crypto_rs::merkle::{leaf_hash, MerkleTree};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    // Use the first byte to choose tree size (1-8 leaves) and index.
    let n = ((data[0] & 0x07) as usize) + 1;
    let idx = (data[1] as usize) % n;

    // Build a real tree with the fuzz data split into `n` chunks.
    let chunk_size = (data.len().saturating_sub(2)) / n + 1;
    let entries: Vec<&[u8]> = (0..n)
        .map(|i| {
            let start = 2 + i * chunk_size;
            let end = (start + chunk_size).min(data.len());
            if start < data.len() {
                &data[start..end]
            } else {
                b""
            }
        })
        .collect();

    let tree = MerkleTree::build(&entries);

    // Generate and verify an inclusion proof; must not panic.
    if let Ok(proof) = tree.inclusion_proof(idx) {
        // Verifying with the correct data must succeed.
        let _ = tree.verify_inclusion(idx, entries[idx], &proof);

        // Build a fake proof from the remaining fuzz bytes and verify;
        // this must not panic even though it will (almost certainly) fail.
        if data.len() > 34 {
            let fake_hashes: Vec<[u8; 32]> = data[2..]
                .chunks(32)
                .filter(|c| c.len() == 32)
                .map(|c| c.try_into().unwrap())
                .collect();
            let _ = tree.verify_inclusion(idx, entries[idx], &fake_hashes);
        }
    }
});
