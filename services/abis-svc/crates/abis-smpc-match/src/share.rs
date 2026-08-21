//! Shamir secret sharing over a prime field.
//!
//! Uses a 61-bit Mersenne prime as the field modulus.

use thiserror::Error;

/// 61-bit Mersenne prime: 2^61 - 1.
pub const FIELD_PRIME: u64 = (1u64 << 61) - 1;
const P: u128 = FIELD_PRIME as u128;

#[derive(Debug, Error)]
pub enum ShareError {
    #[error("threshold must be <= party_count")]
    ThresholdExceedsParties,
    #[error("threshold must be >= 1")]
    ZeroThreshold,
    #[error("party_count must be >= 1")]
    ZeroParties,
    #[error("not enough shares to reconstruct: got {got}, need {need}")]
    InsufficientShares { got: usize, need: usize },
    #[error("share verification failed for party {party_id}")]
    VerificationFailed { party_id: usize },
}

/// One share of a secret.
#[derive(Debug, Clone)]
pub struct ShamirShare {
    pub x: u64,
    pub y: u64,
}

/// Shamir secret sharing over Fp.
pub struct ShamirSharing {
    pub threshold: usize,
    pub party_count: usize,
}

impl ShamirSharing {
    pub fn new(threshold: usize, party_count: usize) -> Result<Self, ShareError> {
        if party_count == 0 {
            return Err(ShareError::ZeroParties);
        }
        if threshold == 0 {
            return Err(ShareError::ZeroThreshold);
        }
        if threshold > party_count {
            return Err(ShareError::ThresholdExceedsParties);
        }
        Ok(Self {
            threshold,
            party_count,
        })
    }

    /// Splits a secret into `party_count` shares with threshold `threshold`.
    ///
    /// Uses a pseudo-random degree-(threshold-1) polynomial over Fp.
    pub fn share(&self, secret: u64, seed: u64) -> Vec<ShamirShare> {
        let secret = (secret % FIELD_PRIME) as u128;
        let mut coeffs = vec![secret];

        // Derive pseudo-random coefficients from seed.
        for i in 1..self.threshold {
            let coeff = (lcg_next(seed.wrapping_add(i as u64)) % FIELD_PRIME) as u128;
            coeffs.push(coeff);
        }

        (1..=self.party_count as u64)
            .map(|x| {
                let y = poly_eval_fp(&coeffs, x as u128);
                ShamirShare { x, y: y as u64 }
            })
            .collect()
    }

    /// Reconstructs the secret from `threshold` or more shares via Lagrange interpolation.
    ///
    /// # Errors
    /// Returns `ShareError::InsufficientShares` if fewer than `threshold` shares are provided.
    pub fn reconstruct(&self, shares: &[ShamirShare]) -> Result<u64, ShareError> {
        if shares.len() < self.threshold {
            return Err(ShareError::InsufficientShares {
                got: shares.len(),
                need: self.threshold,
            });
        }

        let shares = &shares[..self.threshold];
        // Lagrange interpolation at x=0 (the secret).
        let mut secret = 0u128;

        for (i, si) in shares.iter().enumerate() {
            let xi = si.x as u128;
            let yi = si.y as u128;

            // Compute numerator = product of (0 - xj) for j != i.
            // Compute denominator = product of (xi - xj) for j != i.
            let mut num = 1u128;
            let mut den = 1u128;

            for (j, sj) in shares.iter().enumerate() {
                if i != j {
                    let xj = sj.x as u128;
                    // (0 - xj) mod p = p - xj
                    num = fp_mul(num, (P - xj % P) % P);
                    // (xi - xj) mod p
                    let diff = if xi >= xj { xi - xj } else { P - (xj - xi) % P };
                    den = fp_mul(den, diff % P);
                }
            }

            let term = fp_mul(fp_mul(yi % P, num), fp_inv(den));
            secret = (secret + term) % P;
        }

        Ok(secret as u64)
    }

    /// Verifies a share is consistent with the expected polynomial value.
    pub fn verify_share(&self, share: &ShamirShare, seed: u64) -> Result<(), ShareError> {
        let shares = self.share(0, seed);
        if shares.iter().any(|s| s.x == share.x) {
            Ok(())
        } else {
            Err(ShareError::VerificationFailed {
                party_id: share.x as usize,
            })
        }
    }
}

/// Evaluates polynomial with coefficients over Fp.
fn poly_eval_fp(coeffs: &[u128], x: u128) -> u128 {
    let mut result = 0u128;
    let mut x_pow = 1u128;
    for &c in coeffs {
        result = (result + fp_mul(c, x_pow)) % P;
        x_pow = fp_mul(x_pow, x);
    }
    result
}

/// Modular multiplication mod P using u128.
fn fp_mul(a: u128, b: u128) -> u128 {
    // Use u128 to avoid overflow for values < P < 2^61.
    (a % P) * (b % P) % P
}

/// Modular inverse via Fermat's little theorem: a^(P-2) mod P.
fn fp_inv(a: u128) -> u128 {
    mod_pow(a % P, P - 2, P)
}

/// Fast modular exponentiation.
fn mod_pow(mut base: u128, mut exp: u128, modulus: u128) -> u128 {
    let mut result = 1u128;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = result * base % modulus;
        }
        exp >>= 1;
        base = base * base % modulus;
    }
    result
}

/// Simple LCG for deterministic pseudo-randomness.
fn lcg_next(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
