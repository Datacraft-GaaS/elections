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

//! Cross-sensor iris template alignment.

use crate::projection::LinearProjection;
use eel_abis_iris_encode::IRIS_CODE_BYTES;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Sensor wavelength class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SensorType {
    /// Near-infrared, 700-1000 nm.
    Nir,
    /// Visible light, 400-700 nm.
    Visible,
}

/// Errors returned by [`CrossSensorAligner`].
#[derive(Debug, Error)]
pub enum AlignError {
    /// The input code length does not match the projection's expected dimension.
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    /// No training pairs were provided.
    #[error("cannot fit an aligner on empty data")]
    EmptyData,
}

/// Aligns iris templates between two sensor modalities via a learned linear projection.
pub struct CrossSensorAligner {
    /// Learned projection from source to target sensor space.
    pub projection: LinearProjection,
    /// Sensor type of the probe (input).
    pub source_sensor: SensorType,
    /// Sensor type of the gallery (target).
    pub target_sensor: SensorType,
}

impl CrossSensorAligner {
    /// Construct an identity aligner that leaves codes unchanged.
    pub fn identity(dim: usize) -> Self {
        Self {
            projection: LinearProjection::identity(dim),
            source_sensor: SensorType::Nir,
            target_sensor: SensorType::Nir,
        }
    }

    /// Fit a cross-sensor aligner from paired (probe, gallery) code vectors.
    ///
    /// # Errors
    /// Returns [`AlignError::EmptyData`] when `pairs` is empty.
    pub fn fit(
        pairs: &[(Vec<f32>, Vec<f32>)],
        source: SensorType,
        target: SensorType,
    ) -> Result<Self, AlignError> {
        if pairs.is_empty() {
            return Err(AlignError::EmptyData);
        }
        let probe_codes: Vec<Vec<f32>> = pairs.iter().map(|(p, _)| p.clone()).collect();
        let gallery_codes: Vec<Vec<f32>> = pairs.iter().map(|(_, g)| g.clone()).collect();
        let projection = LinearProjection::fit(&probe_codes, &gallery_codes);
        Ok(Self {
            projection,
            source_sensor: source,
            target_sensor: target,
        })
    }

    /// Transform a code from source sensor space to target sensor space.
    ///
    /// # Errors
    /// Returns [`AlignError::DimensionMismatch`] when `code.len()` differs from
    /// the projection's `input_dim`.
    pub fn transform(&self, code: &[f32]) -> Result<Vec<f32>, AlignError> {
        if code.len() != self.projection.input_dim {
            return Err(AlignError::DimensionMismatch {
                expected: self.projection.input_dim,
                actual: code.len(),
            });
        }
        Ok(self.projection.apply(code))
    }
}

/// Convert an IrisCode byte array to a vector of 2048 `f32` bits (0.0 or 1.0).
pub fn iris_code_to_f32(code: &[u8; IRIS_CODE_BYTES]) -> Vec<f32> {
    let mut out = Vec::with_capacity(IRIS_CODE_BYTES * 8);
    for &byte in code.iter() {
        for shift in (0..8).rev() {
            out.push(((byte >> shift) & 1) as f32);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_abis_iris_encode::IRIS_CODE_BITS;

    #[test]
    fn identity_transform_returns_same_code() {
        let aligner = CrossSensorAligner::identity(8);
        let code = vec![0.1, 0.5, 0.9, 0.2, 0.7, 0.3, 0.8, 0.4];
        let out = aligner.transform(&code).expect("transform");
        for (a, b) in out.iter().zip(code.iter()) {
            assert!((a - b).abs() < 1e-5);
        }
    }

    #[test]
    fn fit_on_identical_pairs_near_identity() {
        let pairs: Vec<(Vec<f32>, Vec<f32>)> =
            (0..5).map(|_| (vec![0.5; 8], vec![0.5; 8])).collect();
        let aligner =
            CrossSensorAligner::fit(&pairs, SensorType::Nir, SensorType::Visible).expect("fit");
        let code = vec![0.5; 8];
        let out = aligner.transform(&code).expect("transform");
        for (a, b) in out.iter().zip(code.iter()) {
            assert!((a - b).abs() < 1e-4);
        }
    }

    #[test]
    fn transform_dimension_mismatch_returns_error() {
        let aligner = CrossSensorAligner::identity(8);
        let result = aligner.transform(&[0.0; 4]);
        assert!(matches!(result, Err(AlignError::DimensionMismatch { .. })));
    }

    #[test]
    fn empty_pairs_returns_error() {
        let result = CrossSensorAligner::fit(&[], SensorType::Nir, SensorType::Visible);
        assert!(matches!(result, Err(AlignError::EmptyData)));
    }

    #[test]
    fn sensor_types_serialise_correctly() {
        let nir = SensorType::Nir;
        let vis = SensorType::Visible;
        let j1 = serde_json::to_string(&nir).expect("serialise Nir");
        let j2 = serde_json::to_string(&vis).expect("serialise Visible");
        assert_ne!(j1, j2);
        let back: SensorType = serde_json::from_str(&j1).expect("deserialise Nir");
        assert_eq!(back, SensorType::Nir);
    }

    #[test]
    fn iris_code_to_f32_produces_correct_count() {
        let code = [0u8; IRIS_CODE_BYTES];
        let bits = iris_code_to_f32(&code);
        assert_eq!(bits.len(), IRIS_CODE_BITS);
    }

    #[test]
    fn iris_code_to_f32_values_are_zero_or_one() {
        let code = [0b10110100u8; IRIS_CODE_BYTES];
        let bits = iris_code_to_f32(&code);
        for v in bits {
            assert!(v == 0.0 || v == 1.0, "unexpected value: {v}");
        }
    }

    #[test]
    fn fit_produces_projection_with_correct_dimensions() {
        let pairs: Vec<(Vec<f32>, Vec<f32>)> =
            (0..4).map(|_| (vec![0.0; 16], vec![1.0; 16])).collect();
        let aligner =
            CrossSensorAligner::fit(&pairs, SensorType::Nir, SensorType::Visible).expect("fit");
        assert_eq!(aligner.projection.input_dim, 16);
        assert_eq!(aligner.projection.output_dim, 16);
    }

    #[test]
    fn crosssensor_identity_produces_identity_transform() {
        let aligner = CrossSensorAligner::identity(4);
        let code = vec![1.0, 0.0, 1.0, 0.0];
        let out = aligner.transform(&code).expect("transform");
        assert_eq!(out, code);
    }

    #[test]
    fn align_error_dimension_mismatch_display() {
        let err = AlignError::DimensionMismatch {
            expected: 8,
            actual: 4,
        };
        let msg = err.to_string();
        assert!(msg.contains("8") && msg.contains("4"));
    }

    #[test]
    fn align_error_empty_data_display() {
        let err = AlignError::EmptyData;
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn fit_then_transform_recovers_target_within_tolerance() {
        // Probe codes all-zero, gallery codes all-one: projection adds bias of 1.0.
        let pairs: Vec<(Vec<f32>, Vec<f32>)> =
            (0..10).map(|_| (vec![0.0; 8], vec![1.0; 8])).collect();
        let aligner =
            CrossSensorAligner::fit(&pairs, SensorType::Nir, SensorType::Visible).expect("fit");
        let probe = vec![0.0; 8];
        let out = aligner.transform(&probe).expect("transform");
        for v in out {
            assert!((v - 1.0).abs() < 1e-5);
        }
    }
}
