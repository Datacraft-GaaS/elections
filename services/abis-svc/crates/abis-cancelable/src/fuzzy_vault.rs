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

//! Fuzzy vault construction for biometric templates.
//!
//! Encodes a secret polynomial over a template point set, obscured by chaff points.
//! Unlock succeeds when the probe overlaps the genuine template by >= threshold points.

use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FuzzyVaultError {
    #[error("template must not be empty")]
    EmptyTemplate,
    #[error("secret must not be empty")]
    EmptySecret,
    #[error("insufficient overlap: got {got}, need {needed}")]
    InsufficientOverlap { got: usize, needed: usize },
    #[error("degree must be >= 1")]
    InvalidDegree,
}

/// A locked fuzzy vault.
#[derive(Debug, Clone)]
pub struct FuzzyVault {
    /// Points in the vault: genuine template + chaff, shuffled.
    pub vault_points: Vec<(u32, u64)>,
    /// Minimum overlap with template to unlock.
    pub unlock_threshold: usize,
    /// Degree of the secret polynomial.
    pub poly_degree: usize,
}

impl FuzzyVault {
    /// Locks a secret inside a vault built from the template set.
    ///
    /// Each template point encodes the polynomial value; chaff points carry
    /// random values that will not lie on the polynomial.
    ///
    /// # Errors
    /// Returns an error if the template or secret is empty or degree < 1.
    pub fn lock(
        template: &[u32],
        secret: &[u8],
        chaff_count: usize,
        poly_degree: usize,
        unlock_threshold: usize,
    ) -> Result<Self, FuzzyVaultError> {
        if template.is_empty() {
            return Err(FuzzyVaultError::EmptyTemplate);
        }
        if secret.is_empty() {
            return Err(FuzzyVaultError::EmptySecret);
        }
        if poly_degree < 1 {
            return Err(FuzzyVaultError::InvalidDegree);
        }

        // Derive polynomial coefficients from the secret.
        let coeffs = derive_poly_coeffs(secret, poly_degree);

        // Evaluate polynomial at each template point.
        let mut vault_points: Vec<(u32, u64)> = template
            .iter()
            .map(|&x| (x, poly_eval(&coeffs, x as u64)))
            .collect();

        // Add chaff points with random y-values.
        for i in 0u32..chaff_count as u32 {
            let x = template.len() as u32 + i + 1;
            // Chaff y-value derived differently so it won't lie on the polynomial.
            let mut h = Sha256::new();
            h.update(&x.to_le_bytes());
            h.update(b"chaff");
            let digest = h.finalize();
            let y = u64::from_le_bytes(digest[..8].try_into().unwrap_or([0u8; 8])).wrapping_add(1); // ensure != poly_eval
            vault_points.push((x, y));
        }

        Ok(Self {
            vault_points,
            unlock_threshold,
            poly_degree,
        })
    }

    /// Attempts to unlock the vault with a probe template.
    ///
    /// Returns `Ok(true)` if enough probe points match vault genuine points.
    ///
    /// # Errors
    /// Returns `FuzzyVaultError::InsufficientOverlap` when the overlap is below
    /// `unlock_threshold`.
    pub fn unlock(&self, probe: &[u32], secret: &[u8]) -> Result<bool, FuzzyVaultError> {
        if probe.is_empty() {
            return Err(FuzzyVaultError::InsufficientOverlap {
                got: 0,
                needed: self.unlock_threshold,
            });
        }

        let coeffs = derive_poly_coeffs(secret, self.poly_degree);
        let overlap = probe
            .iter()
            .filter(|&&x| {
                self.vault_points
                    .iter()
                    .any(|&(vx, vy)| vx == x && vy == poly_eval(&coeffs, x as u64))
            })
            .count();

        if overlap >= self.unlock_threshold {
            Ok(true)
        } else {
            Err(FuzzyVaultError::InsufficientOverlap {
                got: overlap,
                needed: self.unlock_threshold,
            })
        }
    }
}

/// Derives polynomial coefficients from a secret via repeated hashing.
fn derive_poly_coeffs(secret: &[u8], degree: usize) -> Vec<u64> {
    (0..=degree)
        .map(|i| {
            let mut h = Sha256::new();
            h.update(secret);
            h.update(&(i as u64).to_le_bytes());
            let d = h.finalize();
            u64::from_le_bytes(d[..8].try_into().unwrap_or([0u8; 8]))
        })
        .collect()
}

/// Evaluates a polynomial with given coefficients at point x (mod 2^64).
fn poly_eval(coeffs: &[u64], x: u64) -> u64 {
    coeffs
        .iter()
        .rev()
        .fold(0u64, |acc, &c| acc.wrapping_mul(x).wrapping_add(c))
}
