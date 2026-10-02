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

//! Shielding parameter selection.
//!
//! Selects Bloom-filter size and hash count to meet target FMR/FNMR
//! under shielded matching.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShieldingParamsError {
    #[error("target_fmr must be in (0, 1)")]
    InvalidFmr,
    #[error("target_fnmr must be in (0, 1)")]
    InvalidFnmr,
    #[error("template_dim_bits must be > 0")]
    ZeroDim,
}

/// Recommended shielding parameters for a given error-rate target.
#[derive(Debug, Clone)]
pub struct ShieldingParams {
    pub filter_size_bits: usize,
    pub hash_count: u8,
    pub target_fmr: f64,
    pub target_fnmr: f64,
}

impl ShieldingParams {
    /// Selects parameters meeting the target FMR/FNMR.
    ///
    /// Uses the standard Bloom-filter optimal formula:
    ///   m = -n * ln(p) / ln(2)^2
    ///   k = m/n * ln(2)
    ///
    /// where n = template_dim_bits, p = target_fmr.
    ///
    /// # Errors
    /// Returns an error if any parameter is out of range.
    pub fn select(
        template_dim_bits: usize,
        target_fmr: f64,
        target_fnmr: f64,
    ) -> Result<Self, ShieldingParamsError> {
        if template_dim_bits == 0 {
            return Err(ShieldingParamsError::ZeroDim);
        }
        if target_fmr <= 0.0 || target_fmr >= 1.0 {
            return Err(ShieldingParamsError::InvalidFmr);
        }
        if target_fnmr <= 0.0 || target_fnmr >= 1.0 {
            return Err(ShieldingParamsError::InvalidFnmr);
        }

        let n = template_dim_bits as f64;
        let ln2_sq = std::f64::consts::LN_2 * std::f64::consts::LN_2;
        let filter_size_bits = ((-n * target_fmr.ln()) / ln2_sq).ceil() as usize;
        let filter_size_bits = filter_size_bits.max(64);

        let k = ((filter_size_bits as f64 / n) * std::f64::consts::LN_2).round() as u8;
        let hash_count = k.clamp(1, 32);

        Ok(Self {
            filter_size_bits,
            hash_count,
            target_fmr,
            target_fnmr,
        })
    }

    /// Returns an estimate of the actual FMR achievable with these parameters.
    pub fn estimated_fmr(&self, template_dim_bits: usize) -> f64 {
        let n = template_dim_bits as f64;
        let m = self.filter_size_bits as f64;
        let k = self.hash_count as f64;
        // Standard Bloom-filter false-positive rate formula.
        (1.0 - (-k * n / m).exp()).powf(k)
    }
}
