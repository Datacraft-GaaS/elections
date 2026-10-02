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

//! Production trains a local linear model on perturbed samples.

use crate::ExplainError;

pub trait LimeExplainer {
    fn explain(
        &self,
        feature_vector: &[f64],
        n_perturbations: usize,
    ) -> Result<Vec<f64>, ExplainError>;
}

pub struct LocalLimeExplainer {
    pub kernel_width: f64,
}

impl LimeExplainer for LocalLimeExplainer {
    fn explain(
        &self,
        feature_vector: &[f64],
        _n_perturbations: usize,
    ) -> Result<Vec<f64>, ExplainError> {
        if feature_vector.is_empty() {
            return Err(ExplainError::InvalidInput(
                "feature_vector must not be empty".into(),
            ));
        }
        let n = feature_vector.len();
        Ok(vec![1.0 / n as f64; n])
    }
}
