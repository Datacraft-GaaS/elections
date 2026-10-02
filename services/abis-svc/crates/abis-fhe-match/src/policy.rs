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

//! FHE policy enforcing security level, noise budget, and circuit depth.

use crate::context::FheContext;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FhePolicyError {
    #[error("context must be provided")]
    MissingContext,
}

/// FHE policy outcome.
#[derive(Debug, Clone)]
pub struct FhePolicyOutcome {
    pub approved: bool,
    pub violations: Vec<String>,
}

/// Enforces FHE operational constraints.
pub struct FhePolicy {
    min_security_bits: u32,
    min_noise_budget: u32,
    max_circuit_depth: u32,
}

impl FhePolicy {
    pub fn new(min_security_bits: u32, min_noise_budget: u32, max_circuit_depth: u32) -> Self {
        Self {
            min_security_bits,
            min_noise_budget,
            max_circuit_depth,
        }
    }

    /// Default policy: 128-bit security, 50 noise budget, 50 depth.
    pub fn default_128bit() -> Self {
        Self::new(128, 50, 50)
    }

    /// Evaluates whether the given context meets the policy.
    ///
    /// # Errors
    /// Currently infallible but returns `Result` for API consistency.
    pub fn check(&self, ctx: &FheContext) -> Result<FhePolicyOutcome, FhePolicyError> {
        let mut violations = Vec::new();

        if ctx.security_level_bits < self.min_security_bits {
            violations.push(format!(
                "security_level_bits {} < minimum {}",
                ctx.security_level_bits, self.min_security_bits
            ));
        }
        if ctx.noise_budget_initial < self.min_noise_budget {
            violations.push(format!(
                "noise_budget_initial {} < minimum {}",
                ctx.noise_budget_initial, self.min_noise_budget
            ));
        }
        if ctx.circuit_depth_max > self.max_circuit_depth {
            violations.push(format!(
                "circuit_depth_max {} > maximum {}",
                ctx.circuit_depth_max, self.max_circuit_depth
            ));
        }

        Ok(FhePolicyOutcome {
            approved: violations.is_empty(),
            violations,
        })
    }
}
