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

//! MCC algorithm parameters.
//!
//! Default values from: Cappelli, Ferrara, Maltoni, IEEE TPAMI 2010, Table I.

use std::f32::consts::PI;

/// Tuning knobs for the Minutia Cylinder-Code algorithm.
#[derive(Debug, Clone)]
pub struct MccParams {
    /// Cylinder radius in pixels (R). Default 70 at 500 ppi.
    pub radius: f32,
    /// Spatial resolution of the cylinder grid (Ns x Ns). Default 16.
    pub ns: u8,
    /// Number of directional layers (Nd). Default 6.
    pub nd: u8,
    /// Standard deviation of the spatial Gaussian. Default R / (3 * Ns) * Ns = R/3 ~ 23.3.
    pub sigma_s: f32,
    /// Standard deviation of the directional Gaussian. Default 2*PI/Nd.
    pub sigma_d: f32,
    /// Psi threshold: cell fires when contribution >= mu_psi. Default 0.001.
    pub mu_psi: f32,
    /// Minimum number of valid (non-zero) cylinders required for a reliable score.
    pub min_matched: u8,
    /// Tau psi: cylinder validity threshold (fraction of non-background cells). Default 0.4.
    pub tau_psi: f32,
}

impl Default for MccParams {
    fn default() -> Self {
        let radius = 70.0_f32;
        let nd = 6u8;
        Self {
            radius,
            ns: 16,
            nd,
            sigma_s: radius / 3.0,
            sigma_d: 2.0 * PI / nd as f32,
            mu_psi: 0.001,
            min_matched: 4,
            tau_psi: 0.4,
        }
    }
}
