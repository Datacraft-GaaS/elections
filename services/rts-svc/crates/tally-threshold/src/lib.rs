//! N6: threshold-encrypted tallies until close.
//!
//! A tally key is generated on the RO desktop and split into `n` Shamir
//! shares handed to trustees; only `k` shares can reconstruct it. Until the
//! polls close, published tallies are sealed under this key so no single
//! party (or fewer than `k` colluding trustees) can read them early. All
//! operations here are PURE: shares, seals, and seals' openings are computed
//! locally from whatever data is at hand, so late/out-of-order/duplicated
//! share batches arriving over intermittent links need no live network at
//! decision time — combine whatever distinct shares you have, whenever they
//! arrive, in any order.
//!
//! # Honesty note (dev-grade construction)
//!
//! This is a coding-grade construction: the stream cipher below is a
//! BLAKE3-derived keystream with a truncated MAC tag, not a vetted AEAD.
//! Production should swap the keystream+tag for a standard AEAD (e.g.
//! AES-GCM or an HChaCha-based construction) behind the same
//! [`seal_payload`] / [`open_payload`] API. The Shamir sharing itself is
//! standard and correct over ark-bls12-381's scalar field.
//!
//! # Combining partial/wrong share sets
//!
//! [`combine_shares`] does NOT authenticate shares: a wrong or partial-but-k
//! set of shares yields *some* 32-byte value, just not the right key.
//! Authentication comes from the payload tag: opening a sealed payload with
//! such a wrong key fails the tag check ([`ThresholdError::TagMismatch`]).

use ark_bls12_381::Fr;
use ark_ff::{BigInteger, Field, One, PrimeField, UniformRand, Zero};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Errors surfaced by threshold splitting/combining and payload sealing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ThresholdError {
    #[error("need at least k={0} distinct shares to combine")]
    TooFewShares(u32),
    #[error("duplicate share id {0}")]
    DuplicateShareId(u32),
    #[error("share id {0} is out of range")]
    BadShareId(u32),
    #[error("share value is not valid hex field bytes")]
    InvalidShareHex,
    #[error("invalid configuration: n={n}, k={k}")]
    InvalidConfig { n: u32, k: u32 },
    #[error("payload authentication failed (wrong key or tampered ciphertext)")]
    TagMismatch,
}

/// One trustee's share: an id in `1..=n` and the scalar as hex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Share {
    pub id: u32,
    pub value_hex: String,
}

/// Sealed payload: XOR-encrypted ciphertext plus a 16-byte authentication tag.
///
/// The tag authenticates the plaintext indirectly through the ciphertext, so
/// any flipped bit (in ciphertext) or wrong reconstruction key fails open().
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedPayload {
    pub ciphertext_hex: String,
    pub tag_hex: String,
}

const MAC_DOMAIN: &[u8] = b"eel-thr-mac";
const ENC_DOMAIN: &[u8] = b"eel-thr-enc";
const TAG_LEN: usize = 16;
const KEY_LEN: usize = 32;

/// Split `key` into `n` shares requiring `k` to reconstruct.
///
/// The 32 key bytes are mapped to a field element; because BLS12-381's scalar
/// field is ~255 bits, a uniform 32-byte string may exceed the modulus. Such
/// candidates are rejected and re-derived as `BLAKE3(key || counter)` for
/// counter = 0, 1, 2, ... until one lands below the modulus (deterministic,
/// so split_key is reproducible apart from the random polynomial).
pub fn split_key(key: &[u8; KEY_LEN], n: u32, k: u32) -> Result<Vec<Share>, ThresholdError> {
    if n == 0 || k == 0 || k > n {
        return Err(ThresholdError::InvalidConfig { n, k });
    }
    let secret = key_to_fr(key);

    // Deterministic RNG seeded from the secret would leak via poly structure;
    // use OS entropy instead. Coefficients a_1..=a_{k-1} of degree-(k-1) poly:
    // f(x) = secret + a_1 x + ... + a_{k-1} x^{k-1}.
    let mut rng = StdRng::from_entropy();
    let mut coeffs = Vec::with_capacity(k as usize);
    for _ in 1..k {
        coeffs.push(Fr::rand(&mut rng));
    }

    let mut shares = Vec::with_capacity(n as usize);
    for id in 1..=n {
        let x = Fr::from(id as u64);
        let mut y = secret;
        let mut xp = x; // running x^j
        for c in &coeffs {
            y += *c * xp;
            xp *= x;
        }
        shares.push(Share {
            id,
            value_hex: fr_to_hex(&y),
        });
    }
    Ok(shares)
}

/// Combine exactly-shaped Lagrange interpolation at 0 from DISTINCT share ids.
///
/// Requires >= k shares with unique ids. NOTE: there is no share-level
/// authentication — a set containing a corrupted share (or simply a different
/// valid subset that doesn't match this secret) produces a well-defined but
/// WRONG key. Callers must authenticate via the payload tag when opening.
pub fn combine_shares(shares: &[Share], k: u32) -> Result<[u8; KEY_LEN], ThresholdError> {
    if shares.len() < k as usize {
        return Err(ThresholdError::TooFewShares(k));
    }
    let mut seen = HashSet::new();
    let mut points = Vec::with_capacity(shares.len());
    for s in shares {
        if s.id == 0 {
            return Err(ThresholdError::BadShareId(0));
        }
        if !seen.insert(s.id) {
            return Err(ThresholdError::DuplicateShareId(s.id));
        }
        let y = fr_from_hex(&s.value_hex)?;
        points.push((Fr::from(s.id as u64), y));
    }

    // Lagrange coefficient L_i(0) = prod_{j != i} x_j / (x_j - x_i).
    let zero = Fr::zero();
    let mut secret = zero;
    for (i, &(xi, yi)) in points.iter().enumerate() {
        let mut num = Fr::one();
        let mut den = Fr::one();
        for (j, &(xj, _)) in points.iter().enumerate() {
            if i == j {
                continue;
            }
            num *= xj;
            den *= xj - xi;
        }
        // den != 0 because ids are distinct nonzero field elements.
        secret += yi * num * den.inverse().unwrap();
    }
    Ok(fr_to_bytes(&secret))
}

/// Seal `plaintext` under `key`.
///
/// Key schedule: `k_mac = BLAKE3("eel-thr-mac" || key)`,
/// `k_enc = BLAKE3("eel-thr-enc" || key)`. Keystream block `i` is
/// `BLAKE3(k_enc || i_u32_le)` (32 bytes per block). Tag =
/// first 16 bytes of `BLAKE3(k_mac || ciphertext)`.
pub fn seal_payload(key: &[u8; KEY_LEN], plaintext: &[u8]) -> SealedPayload {
    let k_mac = blake3::hash(&[MAC_DOMAIN, key].concat());
    let k_enc = blake3::hash(&[ENC_DOMAIN, key].concat());

    let ciphertext = keystream_xor(k_enc.as_bytes(), plaintext);
    let mut hasher = blake3::Hasher::new();
    hasher.update(k_mac.as_bytes());
    hasher.update(&ciphertext);
    let tag_full = hasher.finalize();
    SealedPayload {
        ciphertext_hex: hex::encode(ciphertext),
        tag_hex: hex::encode(&tag_full.as_bytes()[..TAG_LEN]),
    }
}

/// Open a sealed payload: verify tag (near-constant-time compare), decrypt.
pub fn open_payload(
    key: &[u8; KEY_LEN],
    sealed: &SealedPayload,
) -> Result<Vec<u8>, ThresholdError> {
    let k_mac = blake3::hash(&[MAC_DOMAIN, key].concat());
    let k_enc = blake3::hash(&[ENC_DOMAIN, key].concat());

    let ciphertext = hex::decode(&sealed.ciphertext_hex)
        .map_err(|_| ThresholdError::InvalidShareHex)?;
    let tag = hex::decode(&sealed.tag_hex).map_err(|_| ThresholdError::InvalidShareHex)?;

    let mut hasher = blake3::Hasher::new();
    hasher.update(k_mac.as_bytes());
    hasher.update(&ciphertext);
    let expected = hasher.finalize();

    // Constant-ish compare: fold all byte differences, branch once.
    let expect = &expected.as_bytes()[..TAG_LEN.min(expected.as_bytes().len())];
    if tag.len() != expect.len() {
        return Err(ThresholdError::TagMismatch);
    }
    let diff = tag
        .iter()
        .zip(expect.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b));
    if diff != 0 {
        return Err(ThresholdError::TagMismatch);
    }

    Ok(keystream_xor(k_enc.as_bytes(), &ciphertext))
}

// --- internals ---------------------------------------------------------------

/// Map 32 key bytes into Fr, retrying derivation via BLAKE3(key||counter) when
/// the candidate exceeds the field modulus.
fn key_to_fr(key: &[u8; KEY_LEN]) -> Fr {
    if let Some(f) = Fr::from_random_bytes(key) {
        return f;
    }
    for counter in 0u32.. {
        let candidate: [u8; KEY_LEN] = blake3::hash(&[key.as_slice(), &counter.to_le_bytes()].concat())
            .into();
        if let Some(f) = Fr::from_random_bytes(&candidate) {
            return f;
        }
    }
    unreachable!("field-modulus rejection cannot repeat forever")
}

fn fr_to_hex(f: &Fr) -> String {
    hex::encode(fr_to_bytes(f))
}

fn fr_to_bytes(f: &Fr) -> [u8; KEY_LEN] {
    let le = f.into_bigint().to_bytes_le();
    let mut out = [0u8; KEY_LEN];
    out.copy_from_slice(&le[..KEY_LEN]);
    out
}

fn fr_from_hex(hexstr: &str) -> Result<Fr, ThresholdError> {
    let bytes = hex::decode(hexstr.trim()).map_err(|_| ThresholdError::InvalidShareHex)?;
    if bytes.len() > KEY_LEN {
        return Err(ThresholdError::InvalidShareHex);
    }
    let mut limbs = [0u64; 4];
    for (i, chunk) in bytes.chunks(8).enumerate() {
        let mut buf = [0u8; 8];
        buf[..chunk.len()].copy_from_slice(chunk);
        limbs[i] = u64::from_le_bytes(buf);
    }
    Fr::from_bigint(ark_ff::BigInteger256::new(limbs))
        .ok_or(ThresholdError::InvalidShareHex)
}

fn keystream_xor(enc_key: &[u8; KEY_LEN], data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    for (block_idx, chunk) in data.chunks(KEY_LEN).enumerate() {
        let block: [u8; KEY_LEN] =
            blake3::hash(&[enc_key.as_slice(), &(block_idx as u32).to_le_bytes()].concat()).into();
        for (b, k) in chunk.iter().zip(block.iter()) {
            out.push(b ^ k);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key(seed: u8) -> [u8; 32] {
        let mut k = [seed; 32];
        k[31] = seed.wrapping_mul(7);
        k
    }

    #[test]
    fn split_combine_round_trip_any_k_subset() {
        let key = test_key(1);
        let (n, k) = (5u32, 3u32);
        let shares = split_key(&key, n, k).unwrap();

        // Every subset of size k reconstructs the exact key.
        for skip in 0..n as usize {
            let mut subset: Vec<Share> = shares
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .map(|(_, s)| s.clone())
                .collect();
            subset.truncate(k as usize);
            assert_eq!(subset.len(), k as usize);
            assert_eq!(combine_shares(&subset, k).unwrap(), key, "skip={skip}");
        }
    }

    #[test]
    fn n_plus_two_with_k_shares_works() {
        let key = test_key(2);
        let (n, k) = (5u32, 3u32); // n = k + 2
        let shares = split_key(&key, n, k).unwrap();
        // Take shares {2, 4, 5}: arbitrary non-prefix subset.
        let subset: Vec<Share> = vec![shares[1].clone(), shares[3].clone(), shares[4].clone()];
        assert_eq!(combine_shares(&subset, k).unwrap(), key);
    }

    #[test]
    fn duplicate_ids_rejected() {
        let key = test_key(3);
        let shares = split_key(&key, 3, 2).unwrap();
        let duped = vec![shares[0].clone(), shares[0].clone()];
        assert_eq!(
            combine_shares(&duped, 2),
            Err(ThresholdError::DuplicateShareId(duped[0].id))
        );
    }

    #[test]
    fn too_few_shares_rejected() {
        let shares = split_key(&test_key(4), 5, 3).unwrap();
        assert_eq!(
            combine_shares(&shares[..2], 3),
            Err(ThresholdError::TooFewShares(3))
        );
    }

    #[test]
    fn invalid_config_rejected() {
        assert!(split_key(&test_key(5), 3, 5).is_err());
        assert!(split_key(&test_key(5), 0, 0).is_err());
    }

    #[test]
    fn seal_open_round_trip_various_sizes() {
        let key = test_key(6);
        for plaintext in [&b""[..], b"a", b"exactly-32-bytes-long-payload!!!!!!", {
            // >64 bytes
            const BIG: &[u8] = b"the quick brown fox jumps over the lazy dog repeated padding padding padding padding";
            BIG
        }] {
            let sealed = seal_payload(&key, plaintext);
            assert_eq!(open_payload(&key, &sealed).unwrap(), plaintext);
        }
    }

    #[test]
    fn flipped_ciphertext_bit_fails_tag() {
        let key = test_key(7);
        let sealed = seal_payload(&key, b"tally totals pending close");
        let mut ct = hex::decode(&sealed.ciphertext_hex).unwrap();
        ct[0] ^= 0x01; // single flipped bit
        let tampered = SealedPayload {
            ciphertext_hex: hex::encode(ct),
            tag_hex: sealed.tag_hex.clone(),
        };
        assert_eq!(open_payload(&key, &tampered), Err(ThresholdError::TagMismatch));
    }

    #[test]
    fn tampered_share_yields_wrong_key_and_tag_fails_end_to_end() {
        let key = test_key(8);
        let shares = split_key(&key, 4, 2).unwrap();

        // Any k VALID shares reconstruct correctly — even a different subset.
        let other_set = vec![shares[0].clone(), shares[3].clone()];
        assert_eq!(combine_shares(&other_set, 2).unwrap(), key);
        let sealed = seal_payload(&key, b"constituency 34C totals");
        assert!(open_payload(&key, &sealed).is_ok());

        // A literally corrupted share value diverges -> wrong key -> tag failure.
        let mut corrupt = shares[1].clone();
        corrupt.value_hex = hex::encode({
            let mut v = hex::decode(&corrupt.value_hex).unwrap();
            v[0] ^= 0xff;
            v
        });
        let corrupt_key = combine_shares(&vec![shares[0].clone(), corrupt], 2).unwrap();
        assert_ne!(corrupt_key, key);
        assert_eq!(open_payload(&corrupt_key, &sealed), Err(ThresholdError::TagMismatch));
    }

    #[test]
    fn json_serde_round_trip() {
        let shares = split_key(&test_key(9), 3, 2).unwrap();
        let json = serde_json::to_string(&shares).unwrap();
        let back: Vec<Share> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, shares);

        let sealed = seal_payload(&test_key(9), b"x");
        let j2 = serde_json::to_string(&sealed).unwrap();
        assert_eq!(j2, serde_json::to_string(&serde_json::from_str::<SealedPayload>(&j2).unwrap()).unwrap());
    }
}
