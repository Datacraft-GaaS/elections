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

//! FhePolicy: enforces minimum security parameters for the tally circuit.
//!
//! Per CKKS security analysis (Cheon-Kim-Kim-Song ASIACRYPT 2017 and
//! subsequent lattice estimator results):
//!   - security_level >= 128 bits
//!   - poly_modulus_degree >= 8192 for the required circuit depth
//!
//! CI tests use a toy parameter set (degree=32) and pass a test-mode flag.

use thiserror::Error;

use eel_abis_fhe_match::{FheContext, FheScheme};

/// Errors raised when context parameters are below policy minimums.
#[derive(Debug, Error)]
pub enum FhePolicyError {
    #[error("security_level {actual} bits < required {required} bits")]
    InsufficientSecurity { actual: u32, required: u32 },
    #[error("poly_modulus_degree {actual} < required {required} for tally circuit depth")]
    InsufficientDegree { actual: usize, required: usize },
    #[error("scheme must be CKKS for tally, got {0:?}")]
    WrongScheme(FheScheme),
}

/// Production-level tally policy requirements.
pub struct FhePolicy {
    /// Minimum security level in bits. Default: 128.
    pub min_security_level: u32,
    /// Minimum polynomial modulus degree. Default: 8192.
    pub min_poly_degree: usize,
    /// Set true only in test environments with toy parameters.
    pub test_mode: bool,
}

impl Default for FhePolicy {
    fn default() -> Self {
        Self {
            min_security_level: 128,
            min_poly_degree: 8192,
            test_mode: false,
        }
    }
}

impl FhePolicy {
    pub fn test() -> Self {
        Self {
            min_security_level: 1,
            min_poly_degree: 1,
            test_mode: true,
        }
    }

    /// Validate a CkksContext against policy requirements.
    pub fn check(&self, ctx: &FheContext) -> Result<(), FhePolicyError> {
        if self.test_mode {
            return Ok(());
        }
        if ctx.scheme != FheScheme::Ckks {
            return Err(FhePolicyError::WrongScheme(ctx.scheme.clone()));
        }
        if ctx.security_level_bits < self.min_security_level {
            return Err(FhePolicyError::InsufficientSecurity {
                actual: ctx.security_level_bits,
                required: self.min_security_level,
            });
        }
        if ctx.polynomial_degree < self.min_poly_degree {
            return Err(FhePolicyError::InsufficientDegree {
                actual: ctx.polynomial_degree,
                required: self.min_poly_degree,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prod_ctx() -> FheContext {
        FheContext {
            scheme: FheScheme::Ckks,
            polynomial_degree: 8192,
            plaintext_modulus: 0,
            security_level_bits: 128,
            noise_budget_initial: 100,
            circuit_depth_max: 20,
        }
    }

    fn weak_ctx() -> FheContext {
        FheContext {
            scheme: FheScheme::Ckks,
            polynomial_degree: 1024,
            plaintext_modulus: 0,
            security_level_bits: 80,
            noise_budget_initial: 50,
            circuit_depth_max: 5,
        }
    }

    #[test]
    fn production_params_pass() {
        FhePolicy::default().check(&prod_ctx()).unwrap();
    }

    #[test]
    fn weak_security_fails() {
        let err = FhePolicy::default().check(&weak_ctx()).unwrap_err();
        assert!(matches!(err, FhePolicyError::InsufficientSecurity { .. }));
    }

    #[test]
    fn weak_degree_fails() {
        let mut ctx = prod_ctx();
        ctx.polynomial_degree = 4096;
        let err = FhePolicy::default().check(&ctx).unwrap_err();
        assert!(matches!(err, FhePolicyError::InsufficientDegree { .. }));
    }

    #[test]
    fn wrong_scheme_fails() {
        let mut ctx = prod_ctx();
        ctx.scheme = FheScheme::Bfv;
        let err = FhePolicy::default().check(&ctx).unwrap_err();
        assert!(matches!(err, FhePolicyError::WrongScheme(_)));
    }

    #[test]
    fn test_mode_bypasses_checks() {
        FhePolicy::test().check(&weak_ctx()).unwrap();
    }
}
