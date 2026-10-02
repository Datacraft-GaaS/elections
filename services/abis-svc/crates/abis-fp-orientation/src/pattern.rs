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

//! Fingerprint pattern classification from singular points.
//!
//! Reference: D. Maio, D. Maltoni, "Direct gray-scale minutiae detection in fingerprints,"
//! IEEE TPAMI, vol. 19, no. 1, pp. 27-40, 1997. Section on class determination.
//!
//! Classification rules:
//!   Arch:       0 cores, 0 deltas
//!   TentedArch: 0 cores, 1 delta (or 1 core + 1 delta with core above delta)
//!   LeftLoop:   1 core, 1 delta, core to the left of delta
//!   RightLoop:  1 core, 1 delta, core to the right of delta
//!   Whorl:      2 cores, 2 deltas
//!   TwinLoop:   2 cores, 1 delta (or other multi-core configurations)

use crate::singular::{SingularKind, SingularPoint};

/// Fingerprint pattern class (Henry system extended).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FingerprintPattern {
    Arch,
    TentedArch,
    LeftLoop,
    RightLoop,
    Whorl,
    TwinLoop,
}

impl FingerprintPattern {
    /// Returns true when the two patterns are compatible for matching
    /// (same class or adjacent classes).
    pub fn compatible_with(self, other: FingerprintPattern) -> bool {
        use FingerprintPattern::*;
        match (self, other) {
            (a, b) if a == b => true,
            (Arch, TentedArch) | (TentedArch, Arch) => true,
            (LeftLoop, RightLoop) | (RightLoop, LeftLoop) => false,
            (Whorl, TwinLoop) | (TwinLoop, Whorl) => true,
            _ => false,
        }
    }
}

/// Classify a fingerprint from its list of singular points.
pub fn classify_pattern(singular_points: &[SingularPoint]) -> FingerprintPattern {
    let cores: Vec<&SingularPoint> = singular_points
        .iter()
        .filter(|p| matches!(p.kind, SingularKind::Core | SingularKind::DoubleCore))
        .collect();
    let deltas: Vec<&SingularPoint> = singular_points
        .iter()
        .filter(|p| p.kind == SingularKind::Delta)
        .collect();

    match (cores.len(), deltas.len()) {
        (0, 0) => FingerprintPattern::Arch,
        (0, _) | (1, 1) if cores.is_empty() => {
            if deltas.len() == 1 {
                FingerprintPattern::TentedArch
            } else {
                FingerprintPattern::Arch
            }
        }
        (1, 0) => FingerprintPattern::TentedArch,
        (1, 1) => {
            let core = cores[0];
            let delta = deltas[0];
            // Core to the left of delta -> right loop (loop opens right).
            // Core to the right of delta -> left loop.
            if core.x <= delta.x {
                FingerprintPattern::RightLoop
            } else {
                FingerprintPattern::LeftLoop
            }
        }
        (2, _) if deltas.len() >= 2 => FingerprintPattern::Whorl,
        (2, _) => FingerprintPattern::TwinLoop,
        _ => FingerprintPattern::Whorl,
    }
}
