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

use crate::FusionError;

pub fn concatenate(features_a: &[f64], features_b: &[f64]) -> Vec<f64> {
    let mut result = features_a.to_vec();
    result.extend_from_slice(features_b);
    result
}

/// Stub: wire real PCA decomposition in production.
pub fn pca_reduce(features: &[f64], n_components: usize) -> Result<Vec<f64>, FusionError> {
    if n_components > features.len() {
        return Err(FusionError::DimensionMismatch {
            expected: features.len(),
            found: n_components,
        });
    }
    Ok(features[..n_components].to_vec())
}

/// KernelPCA trait: implement compute_kernel_matrix + project; production integrates an ndarray-linalg backend.
pub trait KernelPca {
    fn compute_kernel_matrix(&self, features: &[f64]) -> Vec<Vec<f64>>;
    fn project(&self, kernel_matrix: &[Vec<f64>], n_components: usize) -> Vec<f64>;
}
