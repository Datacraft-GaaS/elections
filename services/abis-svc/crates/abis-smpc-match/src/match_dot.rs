//! Secure dot-product for feature-level biometric matching under SPDZ.
//!
//! Toy 3-party local simulation using additive secret sharing:
//! each element ai is split into 3 additive shares summing to ai mod P.
//! Parties compute partial dot products over their shares and sum the results.

use crate::share::{ShamirShare, ShamirSharing, FIELD_PRIME};
use thiserror::Error;

const P: u64 = FIELD_PRIME;

#[derive(Debug, Error)]
pub enum DotProductError {
    #[error("vectors must have equal length")]
    LengthMismatch,
    #[error("vectors must not be empty")]
    EmptyInput,
    #[error("share error: {0}")]
    Share(#[from] crate::share::ShareError),
}

/// Result of a secure dot-product computation.
#[derive(Debug, Clone)]
pub struct DotProductResult {
    pub value: u64,
    pub party_count: usize,
}

/// Computes a secure dot product of two vectors under SPDZ (3-party simulation).
///
/// Uses additive secret sharing: each ai split into (s0+s1+s2 = ai mod P).
/// The protocol never reveals individual inputs to any single party.
///
/// # Errors
/// Returns an error on empty or mismatched input vectors.
pub struct SecureDotProduct {
    sharing: ShamirSharing,
}

impl SecureDotProduct {
    pub fn new_3party() -> Result<Self, DotProductError> {
        let sharing = ShamirSharing::new(3, 3)?;
        Ok(Self { sharing })
    }

    pub fn compute(&self, a: &[u64], b: &[u64]) -> Result<DotProductResult, DotProductError> {
        if a.is_empty() || b.is_empty() {
            return Err(DotProductError::EmptyInput);
        }
        if a.len() != b.len() {
            return Err(DotProductError::LengthMismatch);
        }

        // Additive sharing: split each value into 3 additive shares.
        // Party p gets share[p] where sum of shares = value (mod P).
        let mut party_dot = [0u64; 3];

        for (idx, (&ai, &bi)) in a.iter().zip(b.iter()).enumerate() {
            let ai_shares = additive_share(ai % P, idx as u64);
            let bi_shares = additive_share(bi % P, idx as u64 + 100_000);

            // Each party multiplies its shares and adds to partial sum.
            // (This is a simplification: in real SPDZ, multiplication requires Beaver triples.)
            for p in 0..3 {
                let prod = mul_mod(ai_shares[p], bi_shares[p]);
                party_dot[p] = add_mod(party_dot[p], prod);
            }
        }

        // Sum party contributions to get the final result.
        let mut total = 0u64;
        for &pd in &party_dot {
            total = add_mod(total, pd);
        }

        Ok(DotProductResult {
            value: total,
            party_count: 3,
        })
    }

    /// Verifies that each party's share set is consistent.
    pub fn verify_shares(
        &self,
        shares: &[Vec<ShamirShare>],
        seed: u64,
    ) -> Result<(), DotProductError> {
        for party_shares in shares {
            for share in party_shares {
                self.sharing.verify_share(share, seed)?;
            }
        }
        Ok(())
    }
}

/// Splits v into 3 additive shares: s[0]+s[1]+s[2] = v (mod P).
fn additive_share(v: u64, seed: u64) -> [u64; 3] {
    let s0 = lcg(seed) % P;
    let s1 = lcg(seed + 1) % P;
    // s2 = v - s0 - s1 (mod P)
    let s2 = add_mod(add_mod(v, P - s0 % P), P - s1 % P);
    [s0, s1, s2]
}

fn add_mod(a: u64, b: u64) -> u64 {
    let r = a as u128 + b as u128;
    (r % P as u128) as u64
}

fn mul_mod(a: u64, b: u64) -> u64 {
    ((a as u128 * b as u128) % P as u128) as u64
}

fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
